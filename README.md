# Monitor Scheduler

Фоновая утилита для Windows и macOS: выключает и включает все мониторы по расписанию.

- **Windows.** Выключение — системный сигнал `SC_MONITORPOWER` (мониторы уходят в режим энергосбережения, ПК продолжает работать). Включение — тот же сигнал плюс микродвижение мыши.
- **macOS.** Выключение — `pmset displaysleepnow` (дисплеи засыпают, Mac продолжает работать). Включение — `caffeinate -u -t 1` (имитация активности пользователя будит дисплеи).

## Сборка

На Windows:

```powershell
./build.ps1
```

Получится `monitors.exe` без консольного окна и две версии для macOS: `monitors-mac-arm64` (Apple Silicon, M1 и новее) и `monitors-mac-amd64` (Intel).

На Mac с установленным Go:

```sh
go build -o monitors .
```

## Установка на macOS

Бинарник, скопированный с другого компьютера, macOS помечает как скачанный. Перед первым запуском:

```sh
chmod +x monitors-mac-arm64
xattr -d com.apple.quarantine monitors-mac-arm64
```

Положите файл в постоянную папку (например, `~/Applications/monitors/`): `config.json`, `monitors.log` и автозапуск привязываются к его расположению.

## Команды

| Команда | Что делает |
|---|---|
| `monitors` или `monitors run` | запускает планировщик |
| `monitors off` | выключить мониторы сейчас (через 1 секунду) |
| `monitors on` | включить мониторы сейчас |
| `monitors install` | добавить планировщик в автозапуск текущего пользователя |
| `monitors uninstall` | убрать из автозапуска |

`monitors` — это `monitors.exe` на Windows и путь к бинарнику на macOS.

Автозапуск на Windows — ключ `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`. На macOS — файл `~/Library/LaunchAgents/com.starcevalexander.monitor-scheduler.plist`; планировщик стартует при следующем входе в систему. Запустить сразу, не перелогиниваясь:

```sh
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.starcevalexander.monitor-scheduler.plist
```

Вывод пишется в `monitors.log` рядом с бинарником.

## Расписание

`config.json` рядом с бинарником (создаётся при первом запуске, пример — `config.example.json`):

```json
{
  "rules": [
    { "action": "off", "time": "23:00" },
    { "action": "on", "time": "08:00", "days": ["mon", "tue", "wed", "thu", "fri"] }
  ]
}
```

- `action` — `off` или `on`
- `time` — `HH:MM`, местное время
- `days` — необязательно: `mon`, `tue`, `wed`, `thu`, `fri`, `sat`, `sun`; без поля правило работает каждый день

Изменения в `config.json` подхватываются на лету, перезапуск не нужен.

## Ограничения

- Мониторы проснутся от движения мыши или нажатия клавиши; повторно сами не гаснут до следующего правила `off`.
- Если компьютер спал во время правила, пропущенное действие не выполняется задним числом.
- Запущен может быть только один планировщик.
- На macOS при включённом запросе пароля после сна дисплея после `on` появится экран входа.
- Остановить на Windows: Диспетчер задач → `monitors.exe` → Снять задачу. На macOS: `launchctl bootout gui/$(id -u)/com.starcevalexander.monitor-scheduler` (если запущен через автозапуск) или `pkill -f monitors`.
