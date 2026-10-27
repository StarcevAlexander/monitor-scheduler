package main

import (
	"context"
	"fmt"
	"io"
	"log"
	"os"
	"os/signal"
	"path/filepath"
	"time"

	"golang.org/x/sys/windows"
)

func main() {
	exePath, err := os.Executable()
	if err != nil {
		log.Fatalf("locate executable: %v", err)
	}
	logFile, err := os.OpenFile(filepath.Join(filepath.Dir(exePath), logFileName), os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0o644)
	if err != nil {
		log.Fatalf("open log: %v", err)
	}
	log.SetOutput(io.MultiWriter(logFile, os.Stderr))
	runErr := run(exePath, os.Args[1:])
	if runErr != nil {
		log.Printf("error: %v", runErr)
	}
	if err := logFile.Close(); err != nil || runErr != nil {
		os.Exit(1)
	}
}

func run(exePath string, args []string) error {
	dir := filepath.Dir(exePath)
	command := "run"
	if len(args) > 0 {
		command = args[0]
	}
	switch command {
	case "off":
		time.Sleep(manualOffDelay)
		return monitorsOff()
	case "on":
		return monitorsOn()
	case "run":
		return runScheduler(filepath.Join(dir, configFileName))
	case "install":
		if err := installAutostart(exePath); err != nil {
			return err
		}
		log.Printf("autostart installed: %s", exePath)
		return nil
	case "uninstall":
		if err := uninstallAutostart(); err != nil {
			return err
		}
		log.Printf("autostart removed")
		return nil
	default:
		return fmt.Errorf("unknown command %q, use: run | on | off | install | uninstall", command)
	}
}

func runScheduler(configPath string) error {
	mutex, err := acquireSingleInstance()
	if err != nil {
		return err
	}
	defer windows.CloseHandle(mutex)

	s, err := newScheduler(configPath, applyAction)
	if err != nil {
		return err
	}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt)
	defer stop()
	log.Printf("scheduler started, config: %s", configPath)
	return s.run(ctx)
}
