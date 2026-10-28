SkyLab 0.1 — internal revision 5. The visible version remains 0.1.

- Untitled project numbers follow open tabs. Returning home and creating a project starts at New project 1 again; internal document identifiers remain stable.
- Distinct update/download and automatic-save icons. Header hover enlarges only the artwork, without expanding or framing its button.
- Source is now a project-layer manager. Imported MIS/MISSION, KML and KMZ files remain separate, expandable groups containing their polygons and lines.
- Right-click a layer to fit or export only that layer as MIS, KML or JSON. Repeated mission names in different layers remain unchanged; combined MIS export requires unique names or separate layer exports.
- KMZ reads its main KML directly from the archive without extracting filesystem paths. Project JSON now uses data format 3 for provenance; existing formats 1 and 2 remain readable. Older application revisions cannot read the new layered JSON format.
- Clicking Satellite while enabled opens the connection selector. Changing XYZ sources keeps the camera and does not require toggling imagery off. Turn imagery off is a separate choice.
- Drag parameter labels horizontally to change values with live previews: right increases, left decreases. No floating slider panel. Numeric typing remains available; Shift allows fine adjustments and Escape cancels an active drag.
- Standardized spacing between the workspace title, editing tools and metrics.

Update inside SkyLab from the prior revision, or close SkyLab and run this installer. Personal files are preserved. Operational validation of exported mission files in the team's navigation system remains necessary before flight use.
