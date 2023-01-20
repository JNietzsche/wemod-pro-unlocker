package main

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"os"
	"path"

	"github.com/google/go-github/v49/github"
	"github.com/shirou/gopsutil/v3/process"
)

func killProcess(name string) error {
	processes, err := process.Processes()
	if err != nil {
		return err
	}
	for _, p := range processes {
		n, err := p.Name()
		if err != nil {
			continue
		}
		if n == name {
			println(n)
			return p.Kill()
		}
	}
	return nil
}

func downloadFile(filepath string, url *string) (err error) {
	os.Remove(filepath)

	out, err := os.Create(filepath)
	if err != nil {
		return err
	}
	defer out.Close()

	resp, err := http.Get(*url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("bad status: %s", resp.Status)
	}

	_, err = io.Copy(out, resp.Body)
	if err != nil {
		return err
	}

	return nil
}

func getLatestReleaseURL() *string {
	client := github.NewClient(nil)
	release, _, err := client.Repositories.GetLatestRelease(context.Background(), "bennett-sh", "wemod-pro-unlocker")
	if err != nil {
		println("error: ", err)
		os.Exit(1)
	}

	assets := release.Assets

	if len(assets) < 1 {
		println("error: no assets in latest release")
		os.Exit(1)
	}

	return assets[0].BrowserDownloadURL
}

func main() {
	if len(os.Args) < 2 {
		println("usage: updater.exe [path-to-cli]")
		os.Exit(1)
	}

	update_program := os.Args[1]

	processName := path.Base(update_program)
	println("\nAttempting to kill WMPU...")
	err := killProcess(processName)
	if err != nil {
		println("failed to kill process:", err)
	}
	println("Done.")

	println("Downloading update...")
	err = downloadFile(update_program, getLatestReleaseURL())
	if err != nil {
		println("error while downloading update:", err)
		os.Exit(1)
	}

	println("Update finished.\n")
}
