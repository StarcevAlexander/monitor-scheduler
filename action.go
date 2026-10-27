package main

import "fmt"

func applyAction(a Action) error {
	switch a {
	case ActionOff:
		return monitorsOff()
	case ActionOn:
		return monitorsOn()
	default:
		return fmt.Errorf("unknown action %q", a)
	}
}
