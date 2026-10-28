SkyLab 0.1 — internal revision 4. The visible version remains 0.1.

- New start screen: create a project, open/import a file or drop it onto the window. Project tabs appear only after a project is opened or created; closing the last tab returns home.
- Startup waits for the logo, fonts and first painted frames before revealing the interface, then fades smoothly.
- About includes credits for the original Delphi developers, contributors and Rafael Dantas (rda@skytem.com).
- Arrow selection tool and scissors cutting tool. Cutting keeps the resulting sections selected with alternating shades.
- Configurable keyboard shortcuts under Help (F1): New Ctrl+N, Open Ctrl+O, Save Ctrl+S, Delete, Select V, Cut Q. Preferences persist on this computer.
- Local recovery snapshot every 90 seconds, with a rotating status indicator. Abnormal termination preserves it and the next launch offers recovery; a normal close clears the recovery cache.
- Optional Google Satellite/Hybrid XYZ connection uses the same endpoint as QGIS QuickMapServices, without an API key. Only visible tiles are requested. Availability and usage conditions depend on the provider. The existing Google Maps Platform connection remains available.

Revision 3 users can update inside SkyLab using Update and restart. Revision 2 or earlier requires this installer once to receive the internal updater. Project files are preserved. Operational validation of exported results in the team's navigation system remains necessary before flight use.
