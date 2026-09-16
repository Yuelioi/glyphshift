# Qt translation service

`windows.qt.translation-service` is the production observation + replacement adapter for the exact
dynamic Qt 6.10.3 / MSVC x64 profile. The explicit `research-qt5` Cargo feature additionally enables
the exact Qt 5.15.13 and Qt 5.15.18 MSVC x64 research profiles; normal production builds reject Qt 5
before installing the hook. The adapter hooks the public
`QCoreApplication::translate(context, sourceText, disambiguation, n)` entry while the complete
UTF-8 source still exists. On a Glyphshift hit it substitutes only the `sourceText` passed to the
original Qt function, so Qt still constructs the returned `QString` and the original function is
called exactly once. On a miss every argument is passed through unchanged.

KeePassXC 2.7.12 supplied the first Qt 5.15.18 ABI sample, but its release build deliberately
restricts its process DACL and rejects the injection-grade process rights used by Glyphshift, so it
is not a representative runtime-validation target. Wireshark 4.6.8 is the representative target:
its Windows package dynamically ships Qt 6.10.3, its executable imports the verified
`QCoreApplication::translate` export, and its main native window title derives from
`tr("The Wireshark Network Analyzer")`. The real-target smoke proves nonzero observation with zero
drops, first- and second-generation visible replacement, and disable recovery. Those title pixels
are drawn by Windows rather than the existing Qt Painter, Qt Quick, or QTextDocument adapters.

Qt 6.10.3 refresh uses Qt's own language-change mechanism: `request_refresh` asynchronously posts a
`QEvent::LanguageChange` to the application, which Qt propagates to top-level windows on their GUI
thread. The Qt 5.15.13 research profile uses the same framework event but a different verified
allocation path: that Qt5Core exports `qMalloc`, whose implementation jumps to its imported UCRT
`malloc`; the `QEvent` scalar deleting destructor and exported `qFree` both release through the same
imported UCRT `free`. The bridge therefore allocates target-owned event storage with `qMalloc`, calls
the public `QEvent(Type)` constructor, then transfers the event to `QCoreApplication::postEvent`.
A real Qt 5.15.13 local lifecycle harness proves a cached menu action and tooltip stay unchanged on a
future translation call, then all retranslate after one posted `LanguageChange`. Qt 5.15.18 still
lacks an independently verified allocation / deletion profile here and remains future-call-only.

The shared text-host V2 event carries `context`, `disambiguation`, and plural `n` through Runtime
capture as observation metadata. Dictionary identity remains source-only: every non-plural Qt call
uses the same source entry regardless of context or disambiguation, while Probe and AI may retain the
metadata as evidence or a translation hint. The Wireshark smoke has real V2 evidence for the same
`Cancel` source under both `SearchFrame` and `WiresharkMainWindow`. A temporary Wireshark startup
preference also drives the real `WelcomePage` numerus source `%n interface(s) shown, %1 hidden` with
`plural_n=7`; plural calls stay observe-only until Glyphshift has an explicit plural-form model.

Before substituting a source, the hook requires the replacement to preserve Qt placeholders (`%n`,
`%Ln`, `%1`…`%99`, including multiplicity; `%%` is treated as a literal percent escape). The real Qt
6.10.3 module contract uses Wireshark's observed `MainStatusBar` source `Profile: %1`: `配置：%1` is
accepted, while `配置` is rejected and the original Qt source is returned. The legacy source-only
decision callback may serve any non-plural call (`n == -1`) because context and disambiguation no
longer participate in Dictionary identity. Plural and unknown negative-`n` calls remain fail-open.

The default Qt 6.10.3 build is included in the production Runtime Bundle after passing the
representative-target context, plural, placeholder, generation-update, and disable gates. The release
Bundle verifier loads it as part of the 17-adapter set. Both Qt 5 profiles remain outside production
and are available only through `research-qt5`. Qt 5.15.13 has the bounded immediate-refresh path
described above; Qt 5.15.18 remains future-call-only until its ownership ABI is independently proven.
