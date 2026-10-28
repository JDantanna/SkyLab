SkyLab 0.2 — internal revision 3.

- Polygon tools now live in a contextual top bar, shown when a polygon is selected on the map or in layers. Live preview and generation use only that polygon. Editing tools also use the top subheader.
- Pen creates closed polygons, including cubic Bézier curves: click for straight anchors, drag for tangent handles, click the first point or press Enter to close. Shift-click a selected outline to insert an anchor; Select moves anchors and handles. Alt separates handles. Escape cancels; draft points support Ctrl+Z/Ctrl+Y.
- Saved JSON format 4 preserves editable curves. Formats 1–3 remain readable. Calculations flatten curves to a 0.1 m tolerance and retain geometric validation. CRS reprojection converts curves to vertices and records a note; KML exports polygon boundaries and holes.
- Import files from the empty workspace, the SkyLab menu or the map context menu. The layer pane focuses on project data.
- XYZ coordinates and Google attribution sit on the bottom map edge, clear of zoom controls. XYZ previews repaint as soon as their calculation finishes.
- Three independent code reviews covered Rust, frontend and Windows integration. Removed an obsolete installer path, overwritten handlers, unused translations/CSS and repeated geometry validation. Corrected deletion after marquee selection and curve-layer export.

Update and restart inside SkyLab, or close SkyLab and run this installer. Personal files are preserved. Mission exports still require operational validation in the team's navigation system.
