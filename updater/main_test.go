package main

import (
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/google/go-github/v49/github"
)

func asset(name, url string) *github.ReleaseAsset {
	return &github.ReleaseAsset{Name: github.String(name), BrowserDownloadURL: github.String(url)}
}

func TestSelectAssetURLPicksNamedAssetRegardlessOfOrder(t *testing.T) {
	assets := []*github.ReleaseAsset{
		asset("updater.exe", "https://x/updater.exe"),
		asset("wemod-pro-unlocker-windows-x64.zip", "https://x/zip"),
		asset("WeMod-Pro-Unlocker.exe", "https://x/cli.exe"),
	}
	got, err := selectAssetURL(assets, defaultAssetName)
	if err != nil {
		t.Fatal(err)
	}
	if got != "https://x/cli.exe" {
		t.Fatalf("got %q", got)
	}
}

func TestSelectAssetURLErrorsWhenMissing(t *testing.T) {
	assets := []*github.ReleaseAsset{
		asset("updater.exe", "https://x/updater.exe"),
		asset("wemod-pro-unlocker-windows-x64.zip", "https://x/zip"),
	}
	_, err := selectAssetURL(assets, defaultAssetName)
	if err == nil || !strings.Contains(err.Error(), "updater.exe") {
		t.Fatalf("expected not-found error listing available assets, got %v", err)
	}
	if _, err := selectAssetURL(nil, defaultAssetName); err == nil {
		t.Fatal("expected error for empty asset list")
	}
}

func TestNormalizeWindowsPath(t *testing.T) {
	cases := map[string]string{
		`\\?\C:\Users\x\wemod-pro-unlocker.exe`: `C:\Users\x\wemod-pro-unlocker.exe`,
		`\\?\UNC\server\share\wmpu.exe`:         `\\server\share\wmpu.exe`,
		`"C:\quoted\wmpu.exe"`:                  `C:\quoted\wmpu.exe`,
		`C:\plain\wmpu.exe`:                     `C:\plain\wmpu.exe`,
	}
	for in, want := range cases {
		if got := normalizeWindowsPath(in); got != want {
			t.Errorf("normalizeWindowsPath(%q) = %q, want %q", in, got, want)
		}
	}
}

func TestProcessNameFromWindowsPath(t *testing.T) {
	if runtime.GOOS != "windows" {
		t.Skip("backslash paths are only separators on Windows")
	}
	for _, in := range []string{
		`C:\Users\Some User\Downloads\wemod-pro-unlocker.exe`,
		`\\?\C:\Users\Some User\Downloads\wemod-pro-unlocker.exe`,
		`C:/Users/Some User/Downloads/wemod-pro-unlocker.exe`,
	} {
		if got := processNameFromPath(in); got != "wemod-pro-unlocker.exe" {
			t.Errorf("processNameFromPath(%q) = %q", in, got)
		}
	}
}

func TestMatchingPIDsIsCaseInsensitiveAndReturnsAll(t *testing.T) {
	procs := []procInfo{
		{pid: 1, name: "WeMod-Pro-Unlocker.exe"},
		{pid: 2, name: "explorer.exe"},
		{pid: 3, name: "wemod-pro-unlocker.exe"},
		{pid: 4, name: "wemod-pro-unlocker.exe"}, // self
	}
	got := matchingPIDs(procs, "wemod-pro-unlocker.exe", 4)
	if len(got) != 2 || got[0] != 1 || got[1] != 3 {
		t.Fatalf("got %v", got)
	}
}

func writeTarget(t *testing.T, content string) (dir, target string) {
	t.Helper()
	dir = t.TempDir()
	target = filepath.Join(dir, "wemod-pro-unlocker.exe")
	if err := os.WriteFile(target, []byte(content), 0o755); err != nil {
		t.Fatal(err)
	}
	return dir, target
}

func assertContent(t *testing.T, path, want string) {
	t.Helper()
	got, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	if string(got) != want {
		t.Fatalf("%s = %q, want %q", path, got, want)
	}
}

func assertOnlyTarget(t *testing.T, dir string) {
	t.Helper()
	entries, err := os.ReadDir(dir)
	if err != nil {
		t.Fatal(err)
	}
	if len(entries) != 1 {
		names := []string{}
		for _, e := range entries {
			names = append(names, e.Name())
		}
		t.Fatalf("expected only the target to remain, found %v", names)
	}
}

func TestDownloadAndReplaceSuccess(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("new binary"))
	}))
	defer srv.Close()

	dir, target := writeTarget(t, "old binary")
	if err := downloadAndReplace(srv.Client(), srv.URL, target); err != nil {
		t.Fatal(err)
	}
	assertContent(t, target, "new binary")
	assertOnlyTarget(t, dir)
}

func TestDownloadAndReplaceKeepsOriginalOnHTTPError(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "nope", http.StatusNotFound)
	}))
	defer srv.Close()

	dir, target := writeTarget(t, "old binary")
	if err := downloadAndReplace(srv.Client(), srv.URL, target); err == nil {
		t.Fatal("expected error")
	}
	assertContent(t, target, "old binary")
	assertOnlyTarget(t, dir)
}

func TestDownloadAndReplaceKeepsOriginalOnEmptyBody(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	dir, target := writeTarget(t, "old binary")
	if err := downloadAndReplace(srv.Client(), srv.URL, target); err == nil {
		t.Fatal("expected error for empty body")
	}
	assertContent(t, target, "old binary")
	assertOnlyTarget(t, dir)
}

func TestDownloadAndReplaceKeepsOriginalWhenServerUnreachable(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {}))
	url := srv.URL
	srv.Close()

	dir, target := writeTarget(t, "old binary")
	if err := downloadAndReplace(http.DefaultClient, url, target); err == nil {
		t.Fatal("expected error")
	}
	assertContent(t, target, "old binary")
	assertOnlyTarget(t, dir)
}

func TestReplaceFileRestoresBackupWhenInstallFails(t *testing.T) {
	_, target := writeTarget(t, "old binary")
	missing := filepath.Join(t.TempDir(), "does-not-exist")

	if err := replaceFile(missing, target); err == nil {
		t.Fatal("expected error")
	}
	assertContent(t, target, "old binary")
	if _, err := os.Stat(target + ".bak"); !os.IsNotExist(err) {
		t.Fatalf("backup should have been restored, stat err = %v", err)
	}
}

func TestReplaceFileWhenTargetDoesNotExist(t *testing.T) {
	dir := t.TempDir()
	src := filepath.Join(dir, "new")
	target := filepath.Join(dir, "wemod-pro-unlocker.exe")
	if err := os.WriteFile(src, []byte("fresh"), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := replaceFile(src, target); err != nil {
		t.Fatal(err)
	}
	assertContent(t, target, "fresh")
}
