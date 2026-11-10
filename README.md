# DownSort

**DownSort** — десктопное приложение на Rust + GPUI для анализа и очистки папки «Загрузки» (Downloads).

Приложение помогает пользователям быстро проанализировать содержимое загрузок, увидеть структуру занятого пространства по типам файлов, обнаружить старые забытые файлы и дубликаты, а также безопасно освободить дисковое пространство через перемещение лишних файлов в системную Корзину.

---

## Архитектура проекта

Проект организован в виде Cargo workspace с разделением на два независимых крейта:

```
DownSort/
├── Cargo.toml               # Манифест Cargo workspace
├── Cargo.lock
├── assets/                  # Иконки и официальные логотипы приложения
├── crates/
│   ├── core/                # Бизнес-логика (сканер, дубликаты, статистика)
│   └── app/                 # Десктопный интерфейс на GPUI (Rust)
├── installer/               # Готовые установщики
│   ├── windows/             # DownSort-Setup.exe, DownSort-Portable.zip, install.bat
│   └── linux/               # install.sh, downsort.desktop, сборка AppImage/.deb
└── README.md                # Документация и руководство пользователя
```

---

## Возможности

1. **Сканирование папки**:
   - Читает папку загрузок по умолчанию (`~/Downloads`) или выбранную директорию.
   - Рекурсивный обход с настраиваемой глубиной (по умолчанию 2 уровня).
   - Извлечение метаданных: имя, полный путь, размер, расширение, дата создания, дата последнего доступа/изменения.
   - Сканирование выполняется асинхронно в фоновом потоке, не блокируя UI.

2. **Категоризация по расширениям**:
   - **Документы**: `doc`, `docx`, `odt`, `rtf`
   - **PDF**: `pdf`
   - **Таблицы**: `xls`, `xlsx`, `csv`, `ods`
   - **Фото**: `jpg`, `jpeg`, `png`, `webp`, `heic`, `gif`, `bmp`
   - **Архивы**: `zip`, `rar`, `7z`, `tar`, `gz`
   - **Установщики**: `exe`, `msi`, `deb`, `rpm`, `AppImage`, `dmg`
   - **Видео/Аудио**: `mp4`, `mkv`, `avi`, `mov`, `mp3`, `wav`, `flac`
   - **Прочее**: все остальные файлы
   - Автоматический подсчёт количества и суммарного объёма для каждой категории.

3. **Поиск дубликатов**:
   - Двухэтапный алгоритм: сначала быстрый размерный фильтр (хешируются только файлы с совпадающим размером), затем многопоточный расчёт SHA-256 (Rayon).
   - В каждой группе дубликатов самый старый файл (по дате создания) помечается как «Оригинал» и **не** выбирается на удаление по умолчанию.
   - Все остальные копии в группе автоматически предвыбираются чекбоксами на удаление.
   - Виртуальная категория «Дубликаты» в сайдбаре отображает счётчик групп и лишних файлов.

4. **Пометка старых файлов**:
   - Файлы, к которым не было обращения более 90 дней, помечаются бейджем «старое (90+ дн)».
   - В верхней сводке и нижней строке выводится суммарный объём, который можно освободить.

5. **Безопасное удаление**:
   - Файлы отправляются в **системную Корзину** (через крейт `trash`), а не удаляются безвозвратно.
   - Перед массовым удалением открывается диалог подтверждения с детальным списком файлов и итоговым размером.
   - Доступно быстрое удаление единичных файлов по иконке корзины в строке файла.

---

## Тестирование бизнес-логики (`core`)

Все ключевые алгоритмы протестированы в `crates/core`:
- Категоризация по расширениям (с проверкой регистра и нестандартных форматов).
- Логика группировки дубликатов на фиктивных данных: правильное выделение оригиналов и предвыбор копий.
- Расчёт старых файлов по дате обращения (>90 дней).
- Арифметика суммирования размеров и подсчёта метрик категорий.

Для запуска тестов:
```bash
cargo test -p core
```

---

## Сборка и запуск

### Windows

Для сборки на Windows требуется Visual Studio (C++ Build Tools / Desktop development with C++) и Windows SDK (для DirectWrite, DirectX и заголовочных файлов платформы).

1. Установите Visual Studio Build Tools с компонентом «Desktop development with C++» (MSVC v14x и Windows 10/11 SDK).
2. Убедитесь, что активен MSVC-тулчейн Rust:
   ```cmd
   rustup default stable-x86_64-pc-windows-msvc
   ```
3. Сборка и запуск в командной строке разработчика (Developer Command Prompt / PowerShell с переменными `vcvars64.bat`):
   ```cmd
   cargo build --release
   cargo run --release -p app
   ```

### Linux

Для сборки на Linux (нативно) требуются системные библиотеки для Wayland/X11, Vulkan, Fontconfig и CMake:

```bash
# Ubuntu / Debian
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    libfontconfig1-dev \
    libx11-dev \
    libx11-xcb-dev \
    libxkbcommon-dev \
    libxkbcommon-x11-dev \
    libwayland-dev \
    libvulkan-dev

# Fedora
sudo dnf install -y \
    gcc gcc-c++ cmake pkg-config \
    fontconfig-devel \
    libX11-devel \
    libxkbcommon-devel \
    libxkbcommon-x11-devel \
    wayland-devel \
    vulkan-loader-devel

# Arch Linux
sudo pacman -S --needed \
    base-devel cmake \
    fontconfig \
    libx11 libxkbcommon libxkbcommon-x11 \
    wayland vulkan-icd-loader
```

Сборка на Linux:
```bash
cargo build --release
```

---

## Установка

### Windows

Доступны два простых способа установки:

1. **Графический установщик (Рекомендуется)**:
   Запустите файл [`installer/windows/DownSort-Setup.exe`](installer/windows/DownSort-Setup.exe). Он автоматически установит программу, добавит значки на Рабочий стол и в меню «Пуск», а также зарегистрирует деинсталлятор в системе.

2. **Портативная версия (без установки)**:
   Распакуйте архив [`installer/windows/DownSort-Portable.zip`](installer/windows/DownSort-Portable.zip) и сразу запустите `DownSort.exe`.

3. **Быстрая установка через скрипт**:
   Запустите [`installer/windows/install.bat`](installer/windows/install.bat).

---

### Linux

1. **Быстрая установка скриптом**:
   ```bash
   chmod +x installer/linux/install.sh
   ./installer/linux/install.sh
   ```
   * Установка под обычным пользователем разворачивает бинарник в `~/.local/bin`, иконки в `~/.local/share/icons` и ярлык приложения в `~/.local/share/applications/downsort.desktop`.
   * При запуске через `sudo ./installer/linux/install.sh` установка выполняется общесистемно в `/usr/local/bin` и `/usr/share/`.

2. **Сборка AppImage или .deb пакета**:
   ```bash
   chmod +x installer/linux/build_appimage.sh installer/linux/build_deb.sh
   ./installer/linux/build_appimage.sh
   ./installer/linux/build_deb.sh
   ```

---

## Статус поддержки платформ (GPUI)

* **Windows**:
  - Поддерживается в ветке GPUI (DirectWrite для шрифтов, DirectX 12 / Blade-graphics для ускоренного рендеринга).
  - Сборка строго требует MSVC (`x86_64-pc-windows-msvc`) и установленного Windows SDK; сборка под GNU (`-pc-windows-gnu` / MinGW) фреймворком GPUI официально не поддерживается из-за несовместимости с ABI DirectX и нативными Windows COM-интерфейсами.

* **Linux**:
  - GPUI активно развивает поддержку Linux на базе `blade-graphics` (Vulkan) и `x11rb` / `wayland-client`.
  - Ограничения на Linux:
    - Требуется поддержка Vulkan драйвером видеокарты (Mesa или проприетарные драйверы NVIDIA/AMD/Intel). В виртуальных машинах без 3D-ускорения Vulkan может потребоваться программный рендерер `lavapipe` (`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json`).
    - Поддержка системного трея и специфичных оконных декораций на некоторых оконных менеджерах (tiling WM / wlroots) может работать в ограниченном режиме по сравнению с macOS и Windows.
