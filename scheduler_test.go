package main

import (
	"testing"
	"time"
)

func newTestScheduler(rules []Rule, applied *[]Action, answers chan time.Duration) *scheduler {
	return &scheduler{
		config: Config{Rules: rules},
		fired: map[firingKey]struct{}{},
		skipped: map[int64]struct{}{},
		postpones: make(chan postponeRequest, postponeQueueSize),
		apply: func(a Action) error {
			*applied = append(*applied, a)
			return nil
		},
		warn: func(time.Duration) (time.Duration, error) {
			return <-answers, nil
		},
	}
}

func receivePostpone(t *testing.T, s *scheduler) postponeRequest {
	t.Helper()
	select {
	case req := <-s.postpones:
		return req
	case <-time.After(time.Second):
		t.Fatal("no postpone request")
		return postponeRequest{}
	}
}

func TestSchedulerPostponesOff(t *testing.T) {
	var applied []Action
	answers := make(chan time.Duration, 2)
	s := newTestScheduler([]Rule{{Action: ActionOff, Time: ClockTime{23, 0}}}, &applied, answers)
	base := time.Date(2026, 10, 26, 22, 59, 10, 0, time.Local)

	answers <- 30 * time.Minute
	s.tick(base)
	req := receivePostpone(t, s)
	if want := base.Add(50 * time.Second); !req.off.Equal(want) || req.delay != 30*time.Minute {
		t.Fatalf("request = %+v, want off %s delay 30m", req, want)
	}
	s.postpone(req)

	s.tick(base.Add(55 * time.Second))
	if len(applied) != 0 {
		t.Fatalf("postponed off still applied at 23:00: %v", applied)
	}

	answers <- 0
	s.tick(time.Date(2026, 10, 26, 23, 29, 5, 0, time.Local))
	select {
	case req := <-s.postpones:
		t.Fatalf("unexpected postpone %+v", req)
	case <-time.After(50 * time.Millisecond):
	}
	if len(applied) != 0 {
		t.Fatalf("applied before deferred time: %v", applied)
	}

	s.tick(time.Date(2026, 10, 26, 23, 30, 5, 0, time.Local))
	if len(applied) != 1 || applied[0] != ActionOff {
		t.Fatalf("applied = %v, want one off at 23:30", applied)
	}
	s.tick(time.Date(2026, 10, 26, 23, 30, 20, 0, time.Local))
	if len(applied) != 1 {
		t.Fatalf("deferred off applied twice: %v", applied)
	}
}

func TestSchedulerPostponesDeferredOffAgain(t *testing.T) {
	var applied []Action
	answers := make(chan time.Duration, 1)
	s := newTestScheduler(nil, &applied, answers)
	off := time.Date(2026, 10, 26, 23, 30, 0, 0, time.Local)
	s.deferred = []deferredOff{{at: off}}

	answers <- 15 * time.Minute
	s.tick(off.Add(-50 * time.Second))
	s.postpone(receivePostpone(t, s))

	s.tick(off.Add(5 * time.Second))
	if len(applied) != 0 {
		t.Fatalf("re-postponed off applied at 23:30: %v", applied)
	}
	if len(s.deferred) != 1 || !s.deferred[0].at.Equal(off.Add(15*time.Minute)) {
		t.Fatalf("deferred = %+v, want single off at 23:45", s.deferred)
	}
}

func TestSchedulerOnRuleNotSkipped(t *testing.T) {
	var applied []Action
	s := newTestScheduler([]Rule{{Action: ActionOn, Time: ClockTime{23, 0}}}, &applied, make(chan time.Duration))
	at := time.Date(2026, 10, 26, 23, 0, 5, 0, time.Local)
	s.skipped[at.Truncate(time.Minute).Unix()] = struct{}{}
	s.tick(at)
	if len(applied) != 1 || applied[0] != ActionOn {
		t.Fatalf("applied = %v, want on", applied)
	}
}
