# QGIS

Status: Qt 5.15.13 research validation in progress

## Target

- QGIS LTR 3.44.12 Windows x64.
- The shipped Windows Qt runtime is Qt 5.15.13 / MSVC x64.
- It is a representative target for a new exact Qt 5.15.13 profile of the generic
  `windows.qt.translation-service` adapter; there are no QGIS-specific production hooks.

## Verified

- The application uses a dynamic Qt5Core and exposes the expected public
  `QCoreApplication::translate` seam.
- A `research-qt5` Glyphshift build can attach to the real application and translate most menu
  strings through that seam.
- Initial manual validation exposed a lifecycle boundary: many already-created nested menu actions
  and tooltips remained English while menus translated elsewhere. A representative example was a
  toolbar tooltip that had been created before the adapter attached.
- A real Qt 5.15.13 local lifecycle reproduction proved the distinction directly: a fresh
  `translate()` call returned the replacement, while text already cached in root / nested menu
  actions and a tooltip stayed unchanged because the research profile did not post
  `LanguageChange`.
- The shipped Qt5Core's binary ABI closes the queue-owned event allocation pair for this exact
  profile: exported `qMalloc` uses its imported UCRT `malloc`, while `QEvent` scalar deletion and
  exported `qFree` use the matching imported UCRT `free`.
- The research adapter now allocates a target-owned `QEvent(LanguageChange)` with `qMalloc` and
  transfers it to `QCoreApplication::postEvent`. The same Qt 5.15.13 reproduction receives one
  `LanguageChange` and retranslates cached root / nested menu actions and the tooltip.
- Real QGIS narrows that lifecycle claim: a publication refresh does produce fresh
  `QCoreApplication::translate` traffic for controls such as `Close`, `Help`, and `Apply`, but the
  cached `New Virtual Layer` QAction / tooltip does not call `translate()` again. The local Qt
  lifecycle harness therefore proves what a cooperative widget can do on `LanguageChange`; it does
  not prove that every application-owned QAction cache participates in retranslation.
- `Qt5Widgets` exposes the standard `QToolTip::showText` overloads, and real QGIS passes the cached
  tooltip through that display-time seam when the user hovers the toolbar action. The value is a
  single-leaf rich-text wrapper (`<b>…</b>` in this case), so the generic Qt Painter adapter now
  translates simple single-leaf tooltip markup at display time while complex / nested markup
  remains fail-open.
- Dictionary remains source-only. Qt `context` / `disambiguation` stay on observation rows and may
  be passed to AI as semantic hints, but they do not create separate Dictionary entries. Render-time
  seams such as Qt Painter therefore use the same source entry after QAction / tooltip caches have
  discarded the original Qt translation metadata.
- The real `New Virtual Layer` tooltip is now visibly translated to `新建虚拟图层`. A stricter run
  removed `windows.qt.translation-service` entirely and kept only Qt Painter plus the required font
  capability; the tooltip still translated while the translation-service observation count did not
  change. This proves the new display-time path independently of startup attachment timing.
- Real nested Vector second / third-level menus are now visibly translated as well. Their final
  labels pass through the existing Qt Painter draw-time seam, so cached QAction text no longer
  depends on QGIS re-entering `QCoreApplication::translate()` after attachment.
- Probe AI planning previously discarded every observation that carried Qt translation context.
  The AI path now accepts those rows and may keep `context/disambiguation` on the selected candidate
  as a translation hint, while candidate dedupe and Dictionary writeback remain source-only. Same
  source across multiple contexts therefore produces one Dictionary entry; plural calls remain
  observe-only. Real QGIS contextual observations now reach the AI path without changing the
  Dictionary schema.
- Default production behavior remains Qt 6.10.3-only. Qt 5.15.13 refresh exists only behind
  `research-qt5`; Qt 5.15.18 remains future-call-only.

## Still to verify on the real application

- Interpret capture drop counters as cumulative for the persisted run. Use per-launch deltas when
  evaluating a change instead of requiring the long-lived counter to return to zero.
- Confirm a second dictionary generation updates already-created controls and disabling the
  workflow restores original text.
- Collect at least one useful contextual duplicate and a real plural call if the application
  naturally exposes them, then verify the same V2 fail-open rules already proven on Qt 6.10.3.

## Resume

Use the synchronized review build with `research-qt5` and the normal full workflow. Toolbar tooltip,
nested menus, and contextual observation / AI-hint propagation are green cases. Resume from a second
source-only Dictionary generation and disable / restore lifecycle without changing the display-time
paths.
