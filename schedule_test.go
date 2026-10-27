package main

import (
	"encoding/json"
	"testing"
	"time"
)

func TestRuleMatches(t *testing.T) {
	monday := time.Date(2026, 10, 26, 23, 0, 30, 0, time.Local)
	tests := []struct {
		name string
		rule Rule
		at time.Time
		want bool
	}{
		{"every day exact minute", Rule{Action: ActionOff, Time: ClockTime{23, 0}}, monday, true},
		{"other minute", Rule{Action: ActionOff, Time: ClockTime{23, 1}}, monday, false},
		{"weekday listed", Rule{Action: ActionOff, Time: ClockTime{23, 0}, Days: []Day{Day(time.Monday)}}, monday, true},
		{"weekday not listed", Rule{Action: ActionOff, Time: ClockTime{23, 0}, Days: []Day{Day(time.Sunday)}}, monday, false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.rule.matches(tt.at); got != tt.want {
				t.Errorf("matches() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestDueRulesFiresOncePerDay(t *testing.T) {
	rules := []Rule{{Action: ActionOff, Time: ClockTime{23, 0}}}
	fired := map[firingKey]struct{}{}
	first := time.Date(2026, 10, 26, 23, 0, 0, 0, time.Local)
	if got := dueRules(rules, first, fired); len(got) != 1 {
		t.Fatalf("first tick: got %d rules, want 1", len(got))
	}
	if got := dueRules(rules, first.Add(30*time.Second), fired); len(got) != 0 {
		t.Fatalf("same minute: got %d rules, want 0", len(got))
	}
	if got := dueRules(rules, first.AddDate(0, 0, 1), fired); len(got) != 1 {
		t.Fatalf("next day: got %d rules, want 1", len(got))
	}
}

func TestConfigJSON(t *testing.T) {
	tests := []struct {
		name string
		input string
		wantErr bool
	}{
		{"valid", `{"rules":[{"action":"off","time":"23:00","days":["mon","Fri"]}]}`, false},
		{"bad action", `{"rules":[{"action":"sleep","time":"23:00"}]}`, true},
		{"bad time", `{"rules":[{"action":"on","time":"25:00"}]}`, true},
		{"bad day", `{"rules":[{"action":"on","time":"08:00","days":["monday"]}]}`, true},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var cfg Config
			err := json.Unmarshal([]byte(tt.input), &cfg)
			if (err != nil) != tt.wantErr {
				t.Fatalf("err = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestDefaultConfigRoundTrip(t *testing.T) {
	data, err := json.Marshal(defaultConfig)
	if err != nil {
		t.Fatal(err)
	}
	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		t.Fatalf("unmarshal %s: %v", data, err)
	}
	if len(cfg.Rules) != len(defaultConfig.Rules) || cfg.Rules[0].Time != defaultConfig.Rules[0].Time {
		t.Fatalf("round trip mismatch: %s", data)
	}
}
