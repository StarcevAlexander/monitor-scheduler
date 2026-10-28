package main

import "time"

const (
	configFileName = "config.json"
	logFileName = "monitors.log"
	checkInterval = 15 * time.Second
	manualOffDelay = time.Second
	dateLayout = "2006-01-02"
	clockLayout = "15:04"
	warningLead = time.Minute
	warningTitle = "Мониторы"
	warningText = "Мониторы выключатся через 1 минуту.\n\nВключить их снова можно только нажав любую кнопку на клавиатуре."
	postponePrompt = "Отложить выключение на:"
	postponeQueueSize = 8
	shortcutName = "Выключить мониторы"
)

var postponeOptions = []postponeOption{
	{label: "15 минут", delay: 15 * time.Minute},
	{label: "30 минут", delay: 30 * time.Minute},
	{label: "1 час", delay: time.Hour},
	{label: "2 часа", delay: 2 * time.Hour},
}

var weekdayNames = map[string]time.Weekday{
	"mon": time.Monday,
	"tue": time.Tuesday,
	"wed": time.Wednesday,
	"thu": time.Thursday,
	"fri": time.Friday,
	"sat": time.Saturday,
	"sun": time.Sunday,
}

var defaultConfig = Config{
	Rules: []Rule{
		{Action: ActionOff, Time: ClockTime{Hour: 23, Minute: 0}},
		{Action: ActionOn, Time: ClockTime{Hour: 8, Minute: 0}},
	},
}
