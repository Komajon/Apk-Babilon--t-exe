# AP_Babilon-T Monitor (Windows .exe)

Интерфейс — прежний HTML (`ui/index.html`), оболочка — Tauri (Rust). Пинг настоящий ICMP (системный `ping.exe`, права администратора не нужны).

## Сборка на GitHub
1. Создайте репозиторий, залейте все файлы этой папки (включая `.github`).
2. Вкладка **Actions** → *Build Windows EXE* → **Run workflow** (или просто push).
3. Готовый `AP_Babilon-T_Monitor.exe` — в **Artifacts** запуска. При теге `v1.0.0` файл появится в **Releases**.

## hosts.json
Положите `hosts.json` рядом с exe — база загрузится при старте (формат как раньше). Без него используется сохранённая локальная база.

Нужен Microsoft Edge WebView2 (есть в Windows 10/11 по умолчанию).
