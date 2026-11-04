# Goal: Pixel-perfect main window after Telegram Desktop (Night)

Status: complete
Source: user instruction 2026-11-04 ("pixel perfect интерфейс для telegram … gpui.rs выглядеть буквально вот так"); reference `artifacts/telegram-donor.png`, defect `artifacts/quill-now.png`; approved plan (copied below)
Last updated: 2026-11-04

## Objective

The main window (folder rail, chat list, chat header, pinned bar, history in wide mode, composer) matches Telegram Desktop's Night theme at 100% scale, 1919×1034. Geometry and colours are 1:1. All metrics come from the tdesktop sources in `artifacts/tdesktop`, not from measuring the picture.

## Execution Directive

Complete the frozen Required Outcomes using the listed Change Envelope and Primary Evidence. Work on the smallest unresolved outcome. Do not add requirements from reviews, tests, tools, speculative risks, or optional source text. Finish when every required outcome is resolved and affected constraints remain satisfied.

## Frozen Contract

### Required Outcomes

Primary evidence for R2–R9: capture the `ready-tdesktop-parity` demo with `scripts/tdesktop_parity/capture.sh`, then compare it to the donor with `scripts/tdesktop_parity/diff.py`. Glyph antialiasing and masked content (avatars, photos) are excluded. Geometry and colours must match.

- **R1: Reference and tooling.**
  - Source: plan, Phase 0.
  - Acceptance:
    - `lib_ui` is fetched.
    - The contract table is written: `docs/decisions/tdesktop-pixel-contract.md`.
    - The `ready-tdesktop-parity` demo reproduces the donor state.
    - The capture and diff scripts run.
  - Primary evidence: the capture PNG plus the diff report.
  - Status: verified
  - Evidence: `lib_ui` at c3f7343; contract in `docs/decisions/tdesktop-pixel-contract.md`; demo `src/ui/tdesktop_parity_demo.rs`; `scripts/tdesktop_parity/{capture.sh,diff.py}` run (final capture of `ready-tdesktop-parity` (nested KWin, 1920×1036) against the donor with `diff.py`: total 2.11%).
- **R2: Foundation.**
  - Source: plan, Phase 1.
  - Acceptance:
    - The licence is GPL-3.0.
    - Open Sans is bundled and is the 13px UI font.
    - tdesktop icon masks are rendered through `td_icon`.
    - The default dark theme is tdesktop `night`.
    - Defaults: folder rail on the left, adaptive wide mode on.
  - Primary evidence: `cargo test` plus the capture.
  - Status: verified
  - Evidence: LICENSE GPL-3.0 (upstream MIT kept in `licenses/`); Open Sans in `assets/fonts`, `Theme::text_size` 13; masks via `src/ui/td/icon.rs`; default night = Tinted; folder rail Left and adaptive wide on, with a one-time prefs move (`move_to_tdesktop_look`). `cargo test --lib` 7139 passed; `cargo test --features demo-capture --bin quill` 2726 passed, 1 load-dependent failure (`settings_window::arrival::every_setting_has_a_place_to_land`, passes 3/3 alone).
- **R3: Window chrome.**
  - Source: plan, Phase 2.
  - Acceptance:
    - The title bar is 24px with tdesktop min/max/close masks and colours.
    - There is no caption, search button or menu bar.
  - Status: verified
  - Evidence: titlebar region 0.00%.
- **R4: Folder rail.**
  - Source: plan, Phase 3.
  - Acceptance:
    - The rail is 72px wide.
    - A 54px hamburger sits on top.
    - Items are 62px, with 36px masks and an 11px semibold label.
    - Colours follow the Night theme.
    - The last item is "Edit".
  - Status: verified
  - Evidence: folder-rail region 1.43% (label glyph AA).
- **R5: Chat list.**
  - Source: plan, Phase 4.
  - Acceptance:
    - The Chats/Contacts/Calls strip is gone.
    - Search is a 35px pill in a 54px zone.
    - Rows are 62px with tdesktop geometry and colours, a channel mask, verified masks and no mute glyph.
    - Dates follow `FormatDialogsDate`.
  - Status: verified
  - Evidence: chat-list region 1.75% (glyph AA, preview mini-thumbs and emoji differ in content).
- **R6: Header and pinned bar.**
  - Source: plan, Phase 5.
  - Acceptance:
    - The header is 54px with icons in the order search, call, info, menu.
    - The status reads "last seen 1 minute ago".
    - The pinned bar is 49px with a pin-list button.
  - Status: verified
  - Evidence: header 0.96%, pinned 2.69% (glyph AA; the donor has a coloured glow under the bar from its own content).
- **R7: History.**
  - Source: plan, Phase 6.
  - Acceptance:
    - The background is `#0E1621`.
    - The wide layout applies at a pane width of 880px or more.
    - Avatar 33px, bubble padding (11,8,11,8), radii 16/6, tail.
    - Bubble colours `#182533` / `#2B5278`.
    - 13px text, tdesktop time and check marks.
  - Status: verified
  - Evidence: history 2.45%; geometry, colours and tail match. Difference: glyph AA plus fixture pictures (photo, reply thumb, GIF) that are not the donor's, the COLRv1 🤖 emoji and the reply emoji pattern.
- **R8: Media in bubbles.**
  - Source: plan, Phase 7.
  - Acceptance:
    - Voice: filled 44px play button, "MM:SS, size" status and a 28×22 transcribe button.
    - Audio file: name plus "00:06".
    - Reply block uses tdesktop metrics.
    - GIF: label and time pills.
  - Status: verified
  - Evidence: Voice, audio, reply and GIF rows match in the history region above; waveforms are the donor's.
- **R9: Composer.**
  - Source: plan, Phase 8.
  - Acceptance:
    - The composer is 54px.
    - Order: attach | field | emoji | mic.
    - There is no "AA" button.
    - The placeholder is "Write a message...".
  - Status: verified
  - Evidence: composer region 1.18%; format button removed (`format_menu_button` deleted).
- **R10: Delivery.**
  - Source: user instruction "реализовать и коммит пуш".
  - Acceptance: the work is committed and pushed to `origin`.
  - Primary evidence: the `git push` output.
  - Status: verified
  - Evidence: `git push origin main` of a45a54e08 and this document update.

### Constraints

- C1: Glyph rasterisation differs between Qt/FreeType and GPUI. Text antialiasing is not required to be bit-identical, but text metrics (size, baseline, width) are.
- C2: The narrow layout (history pane under 880px) must keep working.
- C3: Existing user settings stay switchable. The top folder tabs and the non-wide mode remain available as options.

### Non-goals

- Other screens (settings, dialogs, media viewer) beyond the font change.
- The light theme.

## Change Envelope

- Target: the main window rendering and its defaults.
- Expected paths:
  - `src/ui/{shell.rs,window_frame.rs,folder_tabs.rs,folder_glyphs.rs,chat_list/*,chat_row/*,search_ui/chat_search_bar.rs,conversation/*,message_media/*,synthetic.rs,bubble_header.rs,composer/*,send_button_ui.rs,td_icon.rs}`
  - `src/{chatlist_style.rs,bubble_layout.rs,local_time.rs,voice.rs,settings/appearance.rs,app_theme/*}`
  - `third_party/gpui-component/src/{title_bar.rs,theme/mod.rs,bubble.rs}`
  - `assets/{fonts,tdesktop-icons}/`
  - `LICENSE`, `THIRD_PARTY.md`
- Allowed artifacts:
  - tdesktop icon masks (GPL-3, user decision);
  - Open Sans TTF (OFL);
  - a new demo and scripts under `scripts/tdesktop_parity/`.
- Forbidden: committed screenshots (repo rule).

## Current Checkpoint

- Closes: none (all outcomes verified).

## Current State

- Resolved: R1–R10.
- Last relevant evidence: the final capture diff (total 2.11%; titlebar 0.00, rail 1.43, list 1.75, header 0.96, pinned 2.69, history 2.45, composer 1.18).
- Blocker: none.
- Known leftovers inside C1 or fixture content: glyph AA; the 🤖 emoji (COLRv1, not drawn by GPUI); fixture pictures differ from the donor's.

## Material Decisions

- 2026-11-04: use the tdesktop PNG icon masks, so the fork moves to GPL-3.0 (user).
- 2026-11-04: Open Sans 13px globally (user).

## Checkpoint History

- 2026-11-04: plan approved; `lib_ui` submodule fetched (c3f7343). Values: `fsize` 13px, `windowTitleHeight` 24px, `roundRadiusLarge` 6px, `roundRadiusSmall` 3px.
- 2026-11-04: X11 and desktop Spectacle captures were dropped (CSD insets, locked screen); captures run in a nested headless KWin with a clean config.
- 2026-11-04: R1–R9 verified by capture and diff (91% → 2.11% differing); tests brought in line (a45a54e08).

## Completion

- Resolved outcomes: R1–R10 verified.
- Commands and artifacts: `cargo build --features demo-capture --bin quill`; `scripts/tdesktop_parity/capture.sh target/debug/quill /tmp/quill.png`; `python3 -I scripts/tdesktop_parity/diff.py artifacts/telegram-donor.png /tmp/quill.png /tmp/diff`; `cargo test --lib`; `cargo test --features demo-capture --bin quill`; `cargo clippy --features demo-capture --all-targets` (clean).
- Constraint and diff-scope check: C1 holds (text metrics match, AA differs). C2: the narrow layout tests (`pull_next_ui_tests`, `paragraph_cache_scroll`) pass. C3: top folder tabs and the non-wide mode stay in settings. Changes also touched `src/ui/td/*`, `src/ui/history/*`, `third_party/gpui-pre` (ellipsis) and lang strings, all serving R2–R9.
- Final status: complete.

---

## Approved plan (copy)

See the plan text, mirrored from `/home/stfu/.claude/plans/pixel-kind-dahl.md`, in the appendix below.
## План: pixel-perfect главного окна под Telegram Desktop (Night)

### Context

Эталон: `artifacts/telegram-donor.png` (tdesktop, тема Night, 1919×1034, масштаб 100%). Сейчас Quill выглядит как `artifacts/quill-now.png`. Расхождения:
- есть меню-бар и надпись «Quill» в тайтлбаре, высота тайтлбара 34px;
- сверху стрип Chats/Contacts/Calls и горизонтальные табы папок;
- строки чатов 64px, иконки mute, даты в формате `28 Oct`;
- исходящие сообщения справа, пузыри `#2d333b` и `#1f6feb`;
- в голосовых `0:38` и ссылка «Transcribe»;
- в композере кнопка «AA»;
- системный шрифт 16px.

Цель: главный экран совпадает с эталоном по геометрии и цветам. Все метрики берём из исходников tdesktop в `artifacts/tdesktop`, а не мерим на глаз.

Что важно из разведки: большая часть механизмов уже есть, их надо включить по умолчанию и перевести на точные метрики.
- Вертикальная панель папок: `src/ui/folder_tabs.rs:285` (режим `Left`).
- Wide-раскладка, где исходящие тоже слева: `src/chat_wide.rs:48`, настройка `adaptive_for_wide`, по умолчанию `false` (`src/settings/appearance.rs:245`).
- Палитра tdesktop `night`, вшитая как тема «Tinted»: `src/app_theme/embedded.rs`.
- Тонировка масок: `src/ui/reply_pattern.rs:78` `tinted()`.
- Contacts и Calls уже есть в главном меню: `src/main_menu.rs`.

Решения пользователя: (1) берём PNG-маски иконок из tdesktop, поэтому форк переходит на GPL-3.0; (2) Open Sans 13px глобально.

Честная граница контракта. Геометрия (боксы, отступы, размеры, позиции иконок) и цвета должны совпасть 1:1. Растеризация глифов различается (Qt/FreeType против растеризатора GPUI), поэтому антиалиасинг текста побитово совпадать не будет. Метрики текста (кегль, базовая линия, ширина строки) при этом совпадать обязаны.

### Фаза 0 — Эталон и инструменты

1. Подтянуть пустой сабмодуль: `git -C artifacts/tdesktop submodule update --init Telegram/lib_ui` (коммит c3f7343). Из `lib_ui/ui/basic.style` и `widgets.style` взять:
   - `fsize` и семейства normal/semibold;
   - `defaultWindowTitle`: высоту и кнопки;
   - `roundRadiusLarge` (малый радиус пузыря);
   - hover панели папок, разделитель колонок;
   - стиль линии `MessageBar` для закрепа;
   - шрифты из `lib_ui/fonts`.
2. Записать контракт в `docs/decisions/tdesktop-pixel-contract.md`: таблица «элемент → значение → источник file:line» (основа уже собрана при разведке, см. ниже).
3. Демо `ready-tdesktop-parity`, строится по образцу `apply_ready_showcase` в `src/ui/showcase_demo.rs`, регистрация через `register_demos!` (`src/ui/screenshot_demo.rs`). Состав:
   - папки All chats / Humanoid / Bots / Work / News / AI News (активна) / Info (через `updateChatFolders`);
   - 4 канала с теми же названиями, превью и миниатюрами;
   - чат «Lain»: закреп, текст, два голосовых с ответами, mp3, GIF-ответ;
   - фиксированное «сейчас», чтобы даты были `10:13PM` / `Mon` / `10/28/26`;
   - включённый прокси (щит слева внизу); медиа из `docs/screenshots/fixtures/`.
4. `scripts/pixel_diff.py` (PIL, запуск через `python -I`):
   - снимок 1919×1034 сравнивается с донором по регионам: тайтлбар, панель папок, список, шапка, закреп, история, композер;
   - аватарки и фото маскируются;
   - на выходе heatmap-оверлей и процент расхождения по каждому региону.

   Сборка и запуск: `cargo build --features demo-capture`, затем `QUILL_DEMO_WINDOW_SIZE=1919x1034 QUILL_DEMO_THEME=dark target/debug/quill --screenshot-demo ready-tdesktop-parity <out>`.

### Фаза 1 — Фундамент

- **Лицензия:** `LICENSE` переводим на GPL-3.0. Обновить поле `license` в `Cargo.toml`, README, THIRD_PARTY.md (иконки tdesktop GPL-3, Open Sans OFL), строку про «no Telegram GPL assets» и `licenses/`. `deny.toml` проверяет только крейты, его не трогаем.
- **Шрифт:** Open Sans Regular и Semibold кладём в `assets/fonts/`, регистрируем через `text_system().add_fonts` при старте. В теме ставим `font_family = "Open Sans"` и `font_size = 13` (сейчас 16, `third_party/gpui-component/src/theme/mod.rs:777`). Шрифт сообщений по умолчанию меняем с 14 на 13 (`src/settings/appearance.rs:45`).
- **Иконки:** новый `src/ui/td_icon.rs`:
  - маски @1x/@2x/@3x из `artifacts/tdesktop/Telegram/Resources/icons/` копируются в `assets/tdesktop-icons/` и встраиваются через `include_bytes!`;
  - вариант выбирается по `window.scale_factor()`;
  - маска тонируется через `tinted()` (вынести его из `reply_pattern.rs` в общий модуль);
  - кэш `Arc<RenderImage>` по ключу (иконка, цвет, масштаб);
  - API: `td_icon(TdIcon::DialogsChannel).color(..)`.
- **Тема:** тёмная тема по умолчанию = палитра tdesktop `night` («Tinted») вместо Quill Dark (`src/app_theme/mod.rs:44`). Проверить маппинг `src/ui/app_theme/kit_colors.rs`, добавить недостающие токены: `dialogsBgOver`/`Active`, `sideBar*`, `msgIn/OutBg`, `historyPinnedBg` и т.д.
- **Дефолты настроек:** `folder_tabs_view = Left`, `adaptive_for_wide = true`. Не завязывать дефолты на старые сохранённые значения, у которых нет ключа (тест рядом с `appearance.rs:441`).

### Фаза 2 — Оконный хром

`src/ui/shell.rs:14`, `src/ui/window_frame.rs:295`, `third_party/gpui-component/src/title_bar.rs:16`.
- Высота тайтлбара по `defaultWindowTitle` (в доноре 24px). Фон `#242F3D`, неактивный `#1F2936`.
- Убрать подпись «Quill» и кнопку поиска из тайтлбара.
- Кнопки min/max/close берём из масок `title_button_*`. Цвет иконки `#576673` (hover `#E0E0E0`), hover фона `#2C3847`, close hover `#E92539`.

### Фаза 3 — Панель папок (`src/ui/folder_tabs.rs:430–558`, `src/ui/folder_glyphs.rs`)

- Ширина 72px (сейчас 68), фон `#0E1621`.
- Сверху гамбургер 54px с маской `dialogs_menu` 32×32; переносим его из шапки списка.
- Пункт: min-height 62, маска 36×36 в позиции (−1, 6), подпись 11px semibold на top 40.
- Цвета: подпись и иконка `#768C9E`; активные подпись `#64B9FA`, иконка `#5EB5F7`, фон `#25303E`.
- Убрать левую акцентную полоску и шестерёнку внизу. Последний пункт «Edit», как в tdesktop.
- Маппинг папка → иконка портируем из tdesktop: `ui/filter_icons.style`, логика `ComputeDefaultIcon` и emoticon. Lucide-глифы уходят.
- Бейдж непрочитанного: 17px, обводка 2px.

### Фаза 4 — Колонка списка чатов

`src/ui/chat_list/sidebar.rs`, `header_fit.rs`, `src/ui/search_ui/chat_search_bar.rs:447`, `src/ui/chat_row/mod.rs`, `src/chatlist_style.rs:131`, `src/local_time.rs:251`.
- **Удалить стрип Chats/Contacts/Calls.** Contacts и Calls остаются в главном меню; ветки сайдбара `sidebar.rs:115` открываются оттуда.
- **Ширина колонки:** правило tdesktop с ratio (`window_session_controller.cpp:2806–2833`, мин. 260, макс. 540). В доноре 453px. Убрать `p_3` у колонки.
- **Поиск:** верхняя зона 54px, пилюля высотой 35, радиус 18, отступы слева и справа по 11.
  - Фон `#242F3D`, плейсхолдер `#6D7883`, в фокусе `#4F5B67`.
  - Текстовые отступы (12, 8, 30, 5). Рамка в фокусе 2px `#24303D`.
- **Строка:**
  - Геометрия: высота 62 (в `chatlist_style.rs` и в рендере одновременно), без скругления, на всю ширину. Аватар 46 в (10, 8); имя в (68, 10), semibold 13; превью в (68, 34), 13px.
  - Цвета: имя `#F5F5F5`, превью `#7F91A4`, дата 13px `#8696A8` с правым отступом 10.
  - Фон: hover `#202B36`, выделенная `#2B5278` (вместо `primary.opacity(0.25)`), текст выделенной `#FFFFFF`.
  - Значки: мегафон канала — маска 12×11, смещение (3, 4), отступ 3. Verified — маски star и check (`#6AB3F3` / `#17212B`). Иконку `BellOff` убрать (`mod.rs:698`).
  - Миниатюры в превью: 16×16, радиус 2, отступ справа 3.
  - Счётчик непрочитанных: высота 19, 12px bold, цвета `#4082BC` / `#3E546A`.
- **Даты:** порт `FormatDialogsDate` (`format_values.cpp:709`): сегодня или меньше 20 ч → короткое время локали; меньше 7 дней → день недели; иначе короткая дата локали. Для en_US это `10:13 PM` с U+202F и `10/28/26`. Фиксированные `dd.mm.yy` и `28 Oct` уходят.
- **Кнопка прокси/замка внизу слева:** 36×38, маска щита, видна только при включённом прокси.

### Фаза 5 — Шапка и закреп

`src/ui/conversation/mod.rs:292`, `header_view.rs:103`, `src/ui/message_actions/unpin_from_banner.rs`, `src/telegram/envelope/users.rs:40`.
- **Шапка:**
  - Высота 54, фон `#17212B`. Заголовок semibold 13 на top 8; статус 13px `#708499` (онлайн `#6AB3F3`), на 8px от низа.
  - Убрать левую кнопку сворачивания сайдбара (`mod.rs:302`); шорткат оставить.
  - Кнопки в порядке поиск, звонок, инфо (40×54), меню (44×54). Маски цвета `#6C7883`, hover `#DCDCDC`.
  - Статус пишем по ланг-строкам tdesktop: `last seen 1 minute ago`, `N minutes ago`, `N hours ago`, today/yesterday.
- **Закреп:**
  - Высота 49, фон `#1B2734`. Метка «Pinned message» `#6AB3F3`; линия по стилю `MessageBar` из lib_ui.
  - Справа кнопка 49×49 с маской pin-list, как в tdesktop, и для одиночного закрепа тоже (сейчас там X).

### Фаза 6 — История

`src/bubble_layout.rs`, `src/ui/synthetic.rs:175–566`, `src/ui/conversation/row_chrome.rs`, `src/ui/message_text/mod.rs:474`, `third_party/gpui-component/src/bubble.rs:202`.
- **Фон:** `#0E1621`. Wide-раскладка включается, когда ширина панели ≥ 880.
- **Раскладка:** аватар 33 (сейчас 32), photo skip 40, внешние отступы (16, 6, 56, 2), у пристыкованных top 0.
- **Пузырь:**
  - Паддинги (11, 8, 11, 8), max 430 / min 160.
  - Радиусы: 16 большой, малый по lib_ui. Хвостик рисуем по `ui/chat/message_bubble.cpp`; сейчас вместо хвоста только скругление.
  - Цвета: in `#182533` / out `#2B5278`; текст `#F5F5F5` / `#E4ECF2`; кегль 13.
- **Время и галочки:**
  - Время 13px, `#6D7F8F` / `#7DA8D3`; места под дату 12, delta (2, 5).
  - Галочки — маски `history_sent` / `history_received` цвета `#6BBFFF`.
- Формат времени сообщения `11:03 AM`.

### Фаза 7 — Медиа в пузыре

`src/ui/message_media/audio.rs`, `document.rs:38`, `scrubber.rs`, `src/voice.rs:26`, `src/ui/bubble_header.rs`, `src/ui/reply_pattern.rs`.
- **Голосовое:**
  - Кнопка play 44 заливкой `#3F96D0` / `#4C9CE2` с маской play; паддинги (12, 8, 10, 8).
  - Волна: столбик 2, зазор 1, высота 3–17. Цвета in `#549CD7` / `#3A4D61`, out `#62B2FD` / `#4B7FB3`.
  - Статус `{duration}, {size}`: `MM:SS` с нулями; KB/MB по основанию 1024, усечение до одного знака (`format_values.cpp:54–173`).
  - Кнопка транскрипции 28×22, радиус 4, отступ 10 справа от волны, маска «→A»; ссылка «Transcribe» убирается. Учесть `transcribe_width` в `bubble_width.rs:97`.
- **Аудиофайл:** имя файла и `00:06`, ширина 268–430.
- **Ответ внутри пузыря:**
  - Паддинги (6, 6, 11, 6), полоса 2×36. Цвета in `#429BDB` / out `#65B9F4`, на исходящих не `text_on_fill`.
  - Превью 32; паттерн эмодзи на месте.
- **GIF:** плашка «GIF» в верхнем левом углу, плашка времени фоном `#213040D5` (паддинги 8/2, delta 4), радиус медиа как у пузыря.

### Фаза 8 — Композер

`src/ui/conversation/conversation_part1.rs:157–455`, `src/ui/send_button_ui.rs`, `src/ui/composer/toggle_composer_silent.rs:144`, `field_placeholder.rs`.
- Высота 54 (поле 36 + 2×9), фон `#17212B`, без рамки сверху сверх tdesktop.
- Порядок: attach | поле | emoji | mic/send. Кнопки 44×46, маски `#6C7883`, hover `#DCDCDC`, send `#5288C1`.
- Кнопку «AA» убрать из ряда. Форматирование остаётся через Ctrl+B/I/U и контекстное меню выделения, как в tdesktop.
- Плейсхолдер «Write a message...» с тремя ASCII-точками, как в tdesktop; поправить `bars_tests.rs:120`.

### Verification

1. После каждой фазы: собрать demo-capture, снять `ready-tdesktop-parity` в 1919×1034, прогнать `scripts/pixel_diff.py` против `artifacts/telegram-donor.png` и посмотреть оверлей через Read. Фаза закрыта, когда в её регионе нет геометрических расхождений; остаются только AA глифов и замаскированный контент.
2. Дополнительно снять окно 1200×740 (узкий режим, < 880) и проверить, что не-wide раскладка не сломалась.
3. `cargo test` (поправить тесты на высоту строк, формат дат, плейсхолдер, дефолты настроек), `cargo clippy`, затем `/verify` перед каждым коммитом.
4. Коммитим по фазам и только по команде пользователя.
