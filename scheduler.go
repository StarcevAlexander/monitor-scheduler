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
	skipped map[int64]struct{}
	deferred []deferredOff
	postpones chan postponeRequest
	apply func(Action) error
	warn warnFunc
}

func newScheduler(configPath string, apply func(Action) error, warn warnFunc) (*scheduler, error) {
	s := &scheduler{
		configPath: configPath,
		fired: map[firingKey]struct{}{},
		skipped: map[int64]struct{}{},
		postpones: make(chan postponeRequest, postponeQueueSize),
		apply: apply,
		warn: warn,
	}
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
	s.drainPostpones()
	today := now.Format(dateLayout)
	for key := range s.fired {
		if key.date != today {
			delete(s.fired, key)
		}
	}
	for minute := range s.skipped {
		if now.Unix()-minute >= int64(time.Minute.Seconds()) {
			delete(s.skipped, minute)
		}
	}
	for range dueWarnings(s.config.Rules, now, s.fired) {
		s.startWarning(now.Add(warningLead).Truncate(time.Minute), now)
	}
	for i := range s.deferred {
		d := &s.deferred[i]
		if !d.warned && now.Before(d.at) && !now.Before(d.at.Add(-warningLead)) {
			d.warned = true
			s.startWarning(d.at, now)
		}
	}
	for _, r := range dueRules(s.config.Rules, now, s.fired) {
		if _, skip := s.skipped[now.Truncate(time.Minute).Unix()]; skip && r.Action == ActionOff {
			log.Printf("monitors off (%s) postponed", r.Time)
			continue
		}
		s.runAction(r.Action, r.Time.String())
	}
	pending := s.deferred[:0]
	for _, d := range s.deferred {
		if now.Before(d.at) {
			pending = append(pending, d)
			continue
		}
		s.runAction(ActionOff, d.at.Format(clockLayout))
	}
	s.deferred = pending
}

func (s *scheduler) runAction(a Action, at string) {
	if err := s.apply(a); err != nil {
		log.Printf("%s at %s failed: %v", a, at, err)
		return
	}
	log.Printf("monitors %s (%s)", a, at)
}

func (s *scheduler) startWarning(off, now time.Time) {
	log.Printf("warning: monitors off at %s", off.Format(clockLayout))
	go func() {
		delay, err := s.warn(off.Sub(now))
		if err != nil {
			log.Printf("warning before %s failed: %v", off.Format(clockLayout), err)
			return
		}
		if delay > 0 {
			s.postpones <- postponeRequest{off: off, delay: delay}
		}
	}()
}

func (s *scheduler) drainPostpones() {
	for {
		select {
		case req := <-s.postpones:
			s.postpone(req)
		default:
			return
		}
	}
}

func (s *scheduler) postpone(req postponeRequest) {
	s.skipped[req.off.Unix()] = struct{}{}
	pending := s.deferred[:0]
	for _, d := range s.deferred {
		if !d.at.Equal(req.off) {
			pending = append(pending, d)
		}
	}
	at := req.off.Add(req.delay)
	s.deferred = append(pending, deferredOff{at: at})
	log.Printf("monitors off at %s postponed to %s", req.off.Format(clockLayout), at.Format(clockLayout))
}

func (s *scheduler) run(ctx context.Context) error {
	ticker := time.NewTicker(checkInterval)
	defer ticker.Stop()
	s.tick(time.Now())
	for {
		select {
		case <-ctx.Done():
			return nil
		case req := <-s.postpones:
			s.postpone(req)
		case now := <-ticker.C:
			s.tick(now)
		}
	}
}
