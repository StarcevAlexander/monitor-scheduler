package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"os"
)

func loadConfig(path string) (Config, error) {
	data, err := os.ReadFile(path)
	if errors.Is(err, fs.ErrNotExist) {
		if err := writeConfig(path, defaultConfig); err != nil {
			return Config{}, err
		}
		return defaultConfig, nil
	}
	if err != nil {
		return Config{}, fmt.Errorf("read config: %w", err)
	}
	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return Config{}, fmt.Errorf("parse config %s: %w", path, err)
	}
	if len(cfg.Rules) == 0 {
		return Config{}, fmt.Errorf("config %s: no rules", path)
	}
	return cfg, nil
}

func writeConfig(path string, cfg Config) error {
	data, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return fmt.Errorf("encode config: %w", err)
	}
	if err := os.WriteFile(path, append(data, '\n'), 0o644); err != nil {
		return fmt.Errorf("write config: %w", err)
	}
	return nil
}
