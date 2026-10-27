package main

import (
	"context"
	"fmt"
	"log"
	"os"
	"time"
)

type scheduler struct {
	configPath string
	config Config
	modTime time.Time
	fired map[firingKey]struct{}
	apply func(Action) error
	warn func() error
}

func newScheduler(configPath string, apply func(Action) error, warn func() error) (*scheduler, error) {
	s := &scheduler{configPath: configPath, fired: map[firingKey]struct{}{}, apply: apply, warn: warn}
	if err := s.reload(); err != nil {
		return nil, err
	}
	return s, nil
}

func (s *scheduler) reload() error {
	cfg, err := loadConfig(s.configPath)
	if err != nil {
		return err
	}
	info, err := os.Stat(s.configPath)
	if err != nil {
		return fmt.Errorf("stat config: %w", err)
	}
	s.config = cfg
	s.modTime = info.ModTime()
	for _, r := range cfg.Rules {
		log.Printf("rule: %s at %s days=%v", r.Action, r.Time, r.Days)
	}
	return nil
}

func (s *scheduler) reloadIfChanged() {
	info, err := os.Stat(s.configPath)
	if err != nil || info.ModTime().Equal(s.modTime) {
		return
	}
	log.Printf("config changed, reloading")
	if err := s.reload(); err != nil {
		log.Printf("reload failed, keeping previous rules: %v", err)
		s.modTime = info.ModTime()
	}
}

func (s *scheduler) tick(now time.Time) {
	s.reloadIfChanged()
	today := now.Format(dateLayout)
	for key := range s.fired {
		if key.date != today {
			delete(s.fired, key)
		}
	}
	for _, r := range dueWarnings(s.config.Rules, now, s.fired) {
		log.Printf("warning: monitors off at %s", r.Time)
		go func() {
			if err := s.warn(); err != nil {
				log.Printf("warning before %s failed: %v", r.Time, err)
			}
		}()
	}
	for _, r := range dueRules(s.config.Rules, now, s.fired) {
		if err := s.apply(r.Action); err != nil {
			log.Printf("%s at %s failed: %v", r.Action, r.Time, err)
			continue
		}
		log.Printf("monitors %s (%s)", r.Action, r.Time)
	}
}

func (s *scheduler) run(ctx context.Context) error {
	ticker := time.NewTicker(checkInterval)
	defer ticker.Stop()
	s.tick(time.Now())
	for {
		select {
		case <-ctx.Done():
			return nil
		case now := <-ticker.C:
			s.tick(now)
		}
	}
}
