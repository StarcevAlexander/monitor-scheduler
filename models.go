package main

import "time"

type Action string

const (
	ActionOn Action = "on"
	ActionOff Action = "off"
)

type ClockTime struct {
	Hour int
	Minute int
}

type Day time.Weekday

type Rule struct {
	Action Action `json:"action"`
	Time ClockTime `json:"time"`
	Days []Day `json:"days,omitempty"`
}

type Config struct {
	Rules []Rule `json:"rules"`
}

type firingKey struct {
	date string
	rule int
}
