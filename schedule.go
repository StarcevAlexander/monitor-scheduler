package main

import (
	"encoding/json"
	"fmt"
	"slices"
	"strings"
	"time"
)

func (a *Action) UnmarshalJSON(data []byte) error {
	var s string
	if err := json.Unmarshal(data, &s); err != nil {
		return fmt.Errorf("action: %w", err)
	}
	switch v := Action(strings.ToLower(s)); v {
	case ActionOn, ActionOff:
		*a = v
		return nil
	default:
		return fmt.Errorf("action %q: expected %q or %q", s, ActionOn, ActionOff)
	}
}

func (c ClockTime) String() string {
	return fmt.Sprintf("%02d:%02d", c.Hour, c.Minute)
}

func (c ClockTime) MarshalJSON() ([]byte, error) {
	return json.Marshal(c.String())
}

func (c *ClockTime) UnmarshalJSON(data []byte) error {
	var s string
	if err := json.Unmarshal(data, &s); err != nil {
		return fmt.Errorf("time: %w", err)
	}
	t, err := time.Parse(clockLayout, s)
	if err != nil {
		return fmt.Errorf("time %q: expected HH:MM", s)
	}
	*c = ClockTime{Hour: t.Hour(), Minute: t.Minute()}
	return nil
}

func (d Day) String() string {
	for name, wd := range weekdayNames {
		if wd == time.Weekday(d) {
			return name
		}
	}
	return fmt.Sprintf("day(%d)", int(d))
}

func (d Day) MarshalJSON() ([]byte, error) {
	return json.Marshal(d.String())
}

func (d *Day) UnmarshalJSON(data []byte) error {
	var s string
	if err := json.Unmarshal(data, &s); err != nil {
		return fmt.Errorf("day: %w", err)
	}
	wd, ok := weekdayNames[strings.ToLower(s)]
	if !ok {
		return fmt.Errorf("day %q: expected one of mon, tue, wed, thu, fri, sat, sun", s)
	}
	*d = Day(wd)
	return nil
}

func (r Rule) matches(t time.Time) bool {
	if t.Hour() != r.Time.Hour || t.Minute() != r.Time.Minute {
		return false
	}
	return len(r.Days) == 0 || slices.Contains(r.Days, Day(t.Weekday()))
}

func dueRules(rules []Rule, now time.Time, fired map[firingKey]struct{}) []Rule {
	return collectDue(rules, now, now, false, fired)
}

func dueWarnings(rules []Rule, now time.Time, fired map[firingKey]struct{}) []Rule {
	return collectDue(rules, now, now.Add(warningLead), true, fired)
}

func collectDue(rules []Rule, now, at time.Time, warning bool, fired map[firingKey]struct{}) []Rule {
	date := now.Format(dateLayout)
	var due []Rule
	for i, r := range rules {
		if (warning && r.Action != ActionOff) || !r.matches(at) {
			continue
		}
		key := firingKey{date: date, rule: i, warning: warning}
		if _, done := fired[key]; done {
			continue
		}
		fired[key] = struct{}{}
		due = append(due, r)
	}
	return due
}
