# Установка DownSort / DownSort Installers

Здесь собраны готовые установщики для Windows и Linux.

---

## 🪟 Windows

1. **Обычная установка (в 1 клик)**:
   * Запустите файл **`windows/DownSort-Setup.exe`**.
   * Программа установится в профиль пользователя, появится ярлык на Рабочем столе и в меню «Пуск».
   * Не требует прав администратора!

2. **Портативная версия (без установки)**:
   * Откройте **`windows/DownSort-Portable.zip`**, распакуйте и сразу запускайте `DownSort.exe`.

3. **Установка скриптом**:
   * Запустите **`windows/install.bat`** (или `install.ps1`).

---

## 🐧 Linux (Ubuntu, Debian, Fedora, Arch, Mint и др.)

1. **Быстрая установка одной командой**:
   ```bash
   bash linux/install.sh
   ```
   * Установит бинарник в `~/.local/bin/downsort`
   * Добавит значки и ярлык в системное меню приложений («Уборщик Загрузок»)

2. **Для всей системы (для всех пользователей)**:
   ```bash
   sudo bash linux/install.sh
   ```

3. **Удаление**:
   ```bash
   bash linux/uninstall.sh
   ```

4. **Сборка пакетов AppImage и .deb**:
   ```bash
   bash linux/build_appimage.sh
   bash linux/build_deb.sh
   ```
