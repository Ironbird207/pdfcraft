//! Fill & Sign ▸ Stamps (Add a stamp): the palette and placing stamps on the page.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use pdfcraft_ui_egui::{LeftPanel, PdfCraftApp, QuickTool};

#[test]
fn choosing_and_placing_stamps() {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("form.pdf", None, include_bytes!("data/form.pdf").to_vec()).unwrap();
        app.set_option("zoom", "150").unwrap();
        app
    });
    h.run_steps(6);
    assert!(h.state_mut().execute("comment.stamp"));
    // The panel opens, then scrolls to its stamps.
    h.run_steps(4);
    assert_eq!(h.state().left, LeftPanel::Tool("fill_sign"), "the stamps are part of Fill & Sign");
    h.get_by_label("STAMPS");
    h.get_all_by_label("APPROVED").next().expect("dynamic approved").click();
    h.run_steps(2);
    assert_eq!(h.state().quick_tool, QuickTool::Stamp(pdfcraft_engine::StampKind::DynApproved));
    let p = {
        let r = h.state().views[0].page_screen_rect(0).unwrap();
        r.min + egui::vec2(r.width() * 0.5, r.height() * 0.85)
    };
    h.hover_at(p);
    h.run_steps(1);
    h.drag_at(p);
    h.run_steps(1);
    h.drop_at(p);
    h.run_steps(4);
    let s = h.state();
    let doc = s.session.get(s.views[0].id).unwrap();
    let stamps: Vec<_> = doc.info.annotations.iter().filter(|a| a.subtype == "Stamp").collect();
    assert_eq!(stamps.len(), 1);
    assert_eq!(s.quick_tool, QuickTool::Select, "back to selecting after one stamp");
    for _ in 0..30 {
        h.run_steps(2);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if let Ok(dir) = std::env::var("PDFCRAFT_SHOTS") {
        h.render().unwrap().save(format!("{dir}/stamps.png")).unwrap();
    }
}

#[test]
fn natural_image_stamps_keep_displayed_bounds_on_rotated_pages() {
    for degrees in [0, 90, 180, 270] {
        let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_cc| {
            let mut app = PdfCraftApp::new();
            app.set_option("language", "en").unwrap();
            app.open_bytes("form.pdf", None, include_bytes!("data/form.pdf").to_vec()).unwrap();
            let id = app.views[0].id;
            app.session.apply(id, pdfcraft_engine::Edit::RotatePages { pages: vec![0], degrees }).unwrap();
            app.set_option("zoom", "100").unwrap();
            app
        });
        h.run_steps(6);
        assert!(h.state_mut().execute("comment.stamp"));
        let mut png = Vec::new();
        image::RgbImage::from_pixel(80, 40, image::Rgb([20, 120, 220]))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        h.state_mut().start_custom_stamp("rectangle.png".into(), png);
        h.run_steps(2);
        h.get_by_label("OK").click();
        h.run_steps(2);
        let page = h.state().views[0].page_screen_rect(0).unwrap();
        let at = page.min + egui::vec2(page.width() * 0.5, page.height() * 0.6);
        h.hover_at(at);
        h.run_steps(1);
        h.drag_at(at);
        h.run_steps(1);
        h.drop_at(at);
        h.run_steps(4);
        let app = h.state();
        let doc = app.session.get(app.views[0].id).unwrap();
        assert_eq!(app.quick_tool, QuickTool::Select);
        let stamp = doc.info.annotations.iter().find(|a| a.subtype == "Stamp").unwrap();
        let info = &doc.info.pages[0];
        let a = info.user_to_view(stamp.rect[0], stamp.rect[1]);
        let b = info.user_to_view(stamp.rect[2], stamp.rect[3]);
        assert!(((a[0] - b[0]).abs() - 80.0).abs() < 0.001, "rotation {degrees}: {a:?}, {b:?}");
        assert!(((a[1] - b[1]).abs() - 40.0).abs() < 0.001, "rotation {degrees}: {a:?}, {b:?}");
        assert_eq!(doc.can_undo(), Some("Add stamp"));
    }
}

#[test]
fn creating_placing_and_keeping_custom_stamps() {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("form.pdf", None, include_bytes!("data/form.pdf").to_vec()).unwrap();
        app.set_option("zoom", "150").unwrap();
        app
    });
    h.run_steps(6);
    assert!(h.state_mut().execute("comment.stamp"));
    h.run_steps(2);
    // An image becomes a stamp: Create ▸ category and name.
    let mut png = Vec::new();
    image::RgbImage::from_pixel(60, 20, image::Rgb([20, 120, 220])).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
    h.state_mut().start_custom_stamp("logo.png".into(), png);
    h.run_steps(2);
    h.get_by_label("Create Custom Stamp");
    h.state_mut().stamp_draft.category = "Company".into();
    h.run_steps(1);
    h.get_by_label("OK").click();
    h.run_steps(2);
    assert_eq!(h.state().quick_tool, QuickTool::CustomStamp(0));
    h.get_by_label("Company");
    h.get_by_label("logo");
    // Place it.
    let p = {
        let r = h.state().views[0].page_screen_rect(0).unwrap();
        r.min + egui::vec2(r.width() * 0.5, r.height() * 0.85)
    };
    h.hover_at(p);
    h.run_steps(1);
    h.drag_at(p);
    h.run_steps(1);
    h.drop_at(p);
    h.run_steps(4);
    let s = h.state();
    let doc = s.session.get(s.views[0].id).unwrap();
    let stamps: Vec<_> = doc.info.annotations.iter().filter(|a| a.subtype == "Stamp").collect();
    assert_eq!(stamps.len(), 1);
    let r = stamps[0].rect;
    assert!(((r[2] - r[0]) / (r[3] - r[1]) - 3.0).abs() < 0.05, "keeps the picture's shape: {r:?}");
    assert_eq!(doc.can_undo(), Some("Add stamp"));
    // The library is kept with the app's settings.
    let saved = s.persist();
    let mut again = PdfCraftApp::new();
    again.set_option("language", "en").unwrap();
    again.restore(&saved);
    assert_eq!(again.custom_stamps.len(), 1);
    assert_eq!((again.custom_stamps[0].category.as_str(), again.custom_stamps[0].name.as_str()), ("Company", "logo"));
    assert_eq!(again.custom_stamps[0].data, s.custom_stamps[0].data);
}

/// The Fill & Sign panel, opened by `open` once a document is showing.
fn fill_and_sign_panel(open: impl FnOnce(&mut PdfCraftApp)) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 700.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("form.pdf", None, include_bytes!("data/form.pdf").to_vec()).unwrap();
        app
    });
    h.run_steps(6);
    open(h.state_mut());
    h.run_steps(4);
    h
}

#[test]
fn stamps_are_a_section_of_fill_and_sign() {
    use pdfcraft_engine::catalog;
    assert!(catalog::group("stamp").is_none(), "no separate Add a stamp tool");
    let group = catalog::group("fill_sign").unwrap();
    assert!(group.sections.iter().flat_map(|s| s.items).any(|i| i.command == "comment.stamp"));
    let stamps_top = |h: &Harness<'_, PdfCraftApp>| h.get_by_label("STAMPS").rect().top();
    // Opened as Fill & Sign, the panel starts with the marks and the signatures.
    let plain = fill_and_sign_panel(|app| app.set_option("tool", "fill_sign").unwrap());
    let top = stamps_top(&plain);
    assert!(plain.get_by_label("Checkmark").rect().top() < top);
    assert!(plain.get_by_label("SIGN YOURSELF").rect().top() < top);
    // Add a stamp, and its old tool id, open the same panel scrolled to the stamps.
    let opened = [
        ("Add a stamp", fill_and_sign_panel(|app| assert!(app.execute("comment.stamp")))),
        ("--tool stamp", fill_and_sign_panel(|app| app.set_option("tool", "stamp").unwrap())),
    ];
    for (how, h) in opened {
        assert_eq!(h.state().left, LeftPanel::Tool("fill_sign"), "{how}");
        assert!(h.state().left_open, "{how}");
        let revealed = stamps_top(&h);
        assert!(revealed < top - 150.0, "{how} scrolls to the stamps: {top} -> {revealed}");
    }
}

#[test]
fn the_quick_bar_fill_and_sign_button_leads_to_the_stamps() {
    use egui::accesskit::Toggled;
    let mut h = fill_and_sign_panel(|app| app.set_option("left", "closed").unwrap());
    // A stamp chosen from the palette belongs to the Fill & Sign button, as its marks do: clicking
    // the button opens its pop-out rather than switching to Add text.
    h.state_mut().quick_tool = QuickTool::Stamp(pdfcraft_engine::StampKind::Approved);
    h.run_steps(2);
    assert_eq!(h.get_by_label("Fill & Sign").accesskit_node().toggled(), Some(Toggled::True), "lit while a stamp is chosen");
    h.get_by_label("Fill & Sign").click();
    h.run_steps(2);
    assert_eq!(h.state().quick_tool, QuickTool::Stamp(pdfcraft_engine::StampKind::Approved));
    assert_eq!(h.get_by_label("Stamp palette").accesskit_node().toggled(), Some(Toggled::True));
    h.get_by_label("Stamp palette").click();
    h.run_steps(4);
    assert!(h.state().left_open, "the pop-out opens the tool panel");
    assert_eq!(h.state().left, LeftPanel::Tool("fill_sign"));
    h.get_by_label("STAMPS");
}

/// The stamps share Fill & Sign's panel but not its flattening: Flatten Fill & Sign when saving
/// bakes the marks into the page and leaves review and custom stamps as comments.
#[test]
fn flattening_fill_and_sign_on_save_keeps_stamps_as_comments() {
    use pdfcraft_engine::{Edit, FillMark, MarkFile, NewAnnotation, Shape, StampKind, Style};
    use pdfcraft_ui_egui::SaveTarget;

    let dir = std::env::temp_dir().join(format!("pdfcraft-stamp-flatten-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let add = |shape: Shape| {
        Edit::AddAnnotation(NewAnnotation { page: 0, style: Style::default_for(&shape), shape, contents: String::new(), author: "Ada".into() })
    };
    let mut png = Vec::new();
    image::RgbImage::from_pixel(60, 20, image::Rgb([20, 120, 220])).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
    let mut app = PdfCraftApp::new();
    app.open_bytes("form.pdf", None, include_bytes!("data/form.pdf").to_vec()).unwrap();
    let id = app.views[0].id;
    app.session.apply(id, add(Shape::Mark { rect: [40.0, 80.0, 56.0, 96.0], mark: FillMark::Check })).unwrap();
    app.session.apply(id, add(Shape::Stamp { rect: [40.0, 200.0, 190.0, 240.0], stamp: StampKind::Approved, by: None })).unwrap();
    let file = MarkFile { name: "logo.png".into(), bytes: std::sync::Arc::new(png), page: 0 };
    app.session
        .apply(id, Edit::AddCustomStamp { page: 0, rect: [100.0, 300.0, 100.0, 300.0], name: "logo".into(), file, author: "Ada".into() })
        .unwrap();
    app.flatten_fill_sign_on_save = true;
    let flat = dir.join("flat.pdf");
    app.save_override = Some(flat.to_string_lossy().into_owned());
    assert!(app.save_active(SaveTarget::InPlace));
    let mut reopened = PdfCraftApp::new();
    reopened.open_bytes("flat.pdf", None, std::fs::read(&flat).unwrap()).unwrap();
    let left = &reopened.session.get(reopened.views[0].id).unwrap().info.annotations;
    assert_eq!(left.len(), 2, "the check mark is page content and both stamps stay comments: {left:?}");
    assert!(left.iter().all(|a| a.subtype == "Stamp" && !a.fill_sign), "{left:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_stamp_palette_row_is_left_out_where_the_tool_panel_cannot_show() {
    let mut h = fill_and_sign_panel(|app| {
        app.set_option("mode", "read").unwrap();
        app.quick_tool = QuickTool::Stamp(pdfcraft_engine::StampKind::Approved);
    });
    // Read mode keeps the quick bar but not the tool panel, so the pop-out offers only what works.
    h.get_by_label("Fill & Sign").click();
    h.run_steps(2);
    h.get_by_label("Checkmark");
    assert!(h.query_by_label("Stamp palette").is_none(), "nothing to open in Read mode");
}

#[test]
fn a_scroll_to_the_stamps_that_never_showed_does_not_move_a_later_fill_and_sign() {
    let stamps_top = |h: &Harness<'_, PdfCraftApp>| h.get_by_label("STAMPS").rect().top();
    let plain = fill_and_sign_panel(|app| app.set_option("tool", "fill_sign").unwrap());
    let top = stamps_top(&plain);
    // Asked for in Read mode, where no tool panel shows; then another tool, then back to work.
    let mut h = fill_and_sign_panel(|app| {
        app.set_option("mode", "read").unwrap();
        assert!(app.execute("comment.stamp"));
    });
    h.state_mut().set_option("tool", "edit").unwrap();
    h.state_mut().set_option("mode", "all").unwrap();
    h.run_steps(3);
    h.state_mut().set_option("tool", "fill_sign").unwrap();
    h.run_steps(4);
    assert!((stamps_top(&h) - top).abs() < 1.0, "Fill & Sign opens at its marks: {top} -> {}", stamps_top(&h));
}

#[test]
fn all_tools_does_not_open_where_the_stamps_were_scrolled() {
    let first_tool_top = |h: &Harness<'_, PdfCraftApp>| h.get_by_label("Export a PDF").rect().top();
    let expanded = |app: &mut PdfCraftApp| app.set_option("tools", "expanded").unwrap();
    let fresh = fill_and_sign_panel(expanded);
    let top = first_tool_top(&fresh);
    let mut h = fill_and_sign_panel(|app| {
        expanded(app);
        assert!(app.execute("comment.stamp"));
    });
    // Back to All tools from the stamps.
    h.state_mut().left = LeftPanel::AllTools;
    h.run_steps(4);
    assert!((first_tool_top(&h) - top).abs() < 1.0, "All tools starts at the top: {top} -> {}", first_tool_top(&h));
}

#[test]
fn stamps_stay_inside_a_narrow_fill_and_sign_panel() {
    // The Tools panel at its narrowest; the signature cards above the stamps need more room.
    let narrow = |app: &mut PdfCraftApp| {
        let saved = app.persist();
        let narrowed = saved.replacen("\"FixedFirst\":272.0", "\"FixedFirst\":240.0", 1);
        assert_ne!(narrowed, saved, "the Tools panel's width is in the saved layout: {saved}");
        app.restore(&narrowed);
        app.set_option("tool", "fill_sign").unwrap();
    };
    let h = fill_and_sign_panel(narrow);
    let right = h.get_by_label("Checkmark").rect().right();
    let chip = h.get_all_by_label("APPROVED").next().expect("dynamic approved").rect();
    assert!(chip.right() <= right + 0.5, "stamp rows stay as wide as the Fill rows: {} > {right}", chip.right());
}
