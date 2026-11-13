package main

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/google/go-github/v49/github"
	"github.com/shirou/gopsutil/v3/process"
)

const (
	repoOwner        = "Allain-afk"
	repoName         = "wemod-pro-unlocker"
	defaultAssetName = "wemod-pro-unlocker.exe"

	processExitTimeout = 10 * time.Second
	replaceRetries     = 10
	retryDelay         = 300 * time.Millisecond
)

// normalizeWindowsPath strips the Win32 extended-length prefix that Rust's
// canonicalize() produces (`\\?\C:\...` -> `C:\...`, `\\?\UNC\srv\share` -> `\\srv\share`).
func normalizeWindowsPath(p string) string {
	p = strings.TrimSpace(strings.Trim(p, `"`))
	switch {
	case strings.HasPrefix(p, `\\?\UNC\`):
		return `\\` + p[len(`\\?\UNC\`):]
	case strings.HasPrefix(p, `\\?\`):
		return p[len(`\\?\`):]
	case strings.HasPrefix(p, `//?/`):
		return p[len(`//?/`):]
	}
	return p
}

// processNameFromPath returns the executable file name for a (Windows) path.
func processNameFromPath(p string) string {
	return filepath.Base(normalizeWindowsPath(p))
}

type procInfo struct {
	pid  int32
	name string
}

// matchingPIDs returns the PIDs whose executable name matches name
// (case-insensitively), excluding selfPID.
func matchingPIDs(procs []procInfo, name string, selfPID int32) []int32 {
	var pids []int32
	for _, p := range procs {
		if p.pid != selfPID && strings.EqualFold(p.name, name) {
			pids = append(pids, p.pid)
		}
	}
	return pids
}

func listProcesses() ([]procInfo, error) {
	procs, err := process.Processes()
	if err != nil {
		return nil, err
	}
	infos := make([]procInfo, 0, len(procs))
	for _, p := range procs {
		n, err := p.Name()
		if err != nil {
			continue
		}
		infos = append(infos, procInfo{pid: p.Pid, name: n})
	}
	return infos, nil
}

func findPIDs(name string) ([]int32, error) {
	procs, err := listProcesses()
	if err != nil {
		return nil, err
	}
	return matchingPIDs(procs, name, int32(os.Getpid())), nil
}

// killProcesses terminates every process named name and waits until they
// have all exited (or timeout elapses).
func killProcesses(name string, timeout time.Duration) error {
	pids, err := findPIDs(name)
	if err != nil {
		return err
	}

	var killErrs []error
	for _, pid := range pids {
		fmt.Printf("Stopping %s (PID %d)\n", name, pid)
		p, err := process.NewProcess(pid)
		if err != nil {
			continue // already exited
		}
		if err := p.Kill(); err != nil {
			killErrs = append(killErrs, fmt.Errorf("PID %d: %w", pid, err))
		}
	}

	deadline := time.Now().Add(timeout)
	for {
		remaining, err := findPIDs(name)
		if err != nil {
			return err
		}
		if len(remaining) == 0 {
			return nil
		}
		if time.Now().After(deadline) {
			msgs := make([]string, 0, len(killErrs))
			for _, e := range killErrs {
				msgs = append(msgs, e.Error())
			}
			return fmt.Errorf("%s still running (PIDs %v) after %s: %s",
				name, remaining, timeout, strings.Join(msgs, "; "))
		}
		time.Sleep(200 * time.Millisecond)
	}
}

// selectAssetURL returns the download URL of the release asset named name.
func selectAssetURL(assets []*github.ReleaseAsset, name string) (string, error) {
	if len(assets) == 0 {
		return "", errors.New("no assets in latest release")
	}
	available := make([]string, 0, len(assets))
	for _, asset := range assets {
		if strings.EqualFold(asset.GetName(), name) && asset.GetBrowserDownloadURL() != "" {
			return asset.GetBrowserDownloadURL(), nil
		}
		available = append(available, asset.GetName())
	}
	return "", fmt.Errorf("asset %q not found in latest release (available: %s)",
		name, strings.Join(available, ", "))
}

func getLatestReleaseURL(assetName string) (string, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	client := github.NewClient(nil)
	release, _, err := client.Repositories.GetLatestRelease(ctx, repoOwner, repoName)
	if err != nil {
		return "", fmt.Errorf("failed to fetch latest release: %w", err)
	}
	return selectAssetURL(release.Assets, assetName)
}

// downloadToTemp downloads url into a new temporary file inside dir and
// returns its path. The file is fully written and closed on success; on
// failure it is removed.
func downloadToTemp(client *http.Client, url, dir string) (_ string, err error) {
	resp, err := client.Get(url)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("bad status: %s", resp.Status)
	}

	tmp, err := os.CreateTemp(dir, ".wemod-pro-unlocker-*.download")
	if err != nil {
		return "", err
	}
	tmpPath := tmp.Name()
	defer func() {
		if err != nil {
			tmp.Close()
			os.Remove(tmpPath)
		}
	}()

	n, err := io.Copy(tmp, resp.Body)
	if err != nil {
		return "", err
	}
	if n == 0 {
		return "", errors.New("downloaded file is empty")
	}
	if resp.ContentLength > 0 && n != resp.ContentLength {
		return "", fmt.Errorf("incomplete download: got %d of %d bytes", n, resp.ContentLength)
	}
	if err = tmp.Sync(); err != nil {
		return "", err
	}
	if err = tmp.Close(); err != nil {
		return "", err
	}
	return tmpPath, nil
}

func renameWithRetry(from, to string) error {
	var err error
	for i := 0; i < replaceRetries; i++ {
		if err = os.Rename(from, to); err == nil {
			return nil
		}
		time.Sleep(retryDelay)
	}
	return err
}

// replaceFile atomically-ish swaps newPath into target. The existing target is
// first moved to target.bak and restored if the swap fails.
func replaceFile(newPath, target string) error {
	backup := target + ".bak"
	hadTarget := false

	if _, err := os.Stat(target); err == nil {
		hadTarget = true
		os.Remove(backup)
		if err := renameWithRetry(target, backup); err != nil {
			return fmt.Errorf("failed to back up %s: %w", target, err)
		}
	} else if !errors.Is(err, os.ErrNotExist) {
		return err
	}

	if err := renameWithRetry(newPath, target); err != nil {
		if hadTarget {
			if rbErr := renameWithRetry(backup, target); rbErr != nil {
				return fmt.Errorf("failed to install update (%v) and failed to restore backup %s: %w", err, backup, rbErr)
			}
		}
		return fmt.Errorf("failed to install update: %w", err)
	}

	if hadTarget {
		os.Remove(backup) // best effort
	}
	return nil
}

// downloadAndReplace downloads url and replaces target with it. target is
// left untouched if the download fails.
func downloadAndReplace(client *http.Client, url, target string) error {
	tmp, err := downloadToTemp(client, url, filepath.Dir(target))
	if err != nil {
		return err
	}
	if err := replaceFile(tmp, target); err != nil {
		os.Remove(tmp)
		return err
	}
	return nil
}

func run(args []string) error {
	if len(args) < 2 {
		return errors.New("usage: updater.exe [path-to-cli]")
	}

	target, err := filepath.Abs(normalizeWindowsPath(args[1]))
	if err != nil {
		return fmt.Errorf("invalid target path: %w", err)
	}
	processName := processNameFromPath(target)

	fmt.Println("\nAttempting to stop WMPU...")
	if err := killProcesses(processName, processExitTimeout); err != nil {
		fmt.Println("warning: failed to stop process:", err)
	}
	fmt.Println("Done.")

	fmt.Println("Downloading update...")
	url, err := getLatestReleaseURL(defaultAssetName)
	if err != nil {
		return err
	}

	client := &http.Client{Timeout: 5 * time.Minute}
	if err := downloadAndReplace(client, url, target); err != nil {
		return fmt.Errorf("error while downloading update: %w", err)
	}

	fmt.Println("Update finished. Run wemod-pro-unlocker.exe again to patch.")
	return nil
}

func main() {
	if err := run(os.Args); err != nil {
		fmt.Println("error:", err)
		os.Exit(1)
	}
}
