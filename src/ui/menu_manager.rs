use gtk4::prelude::*;
use gtk4::{gio, MenuButton, PopoverMenu};
use std::cell::RefCell;
use std::rc::Rc;
use crate::ui::MainWindow;

/// Creates all menus for the application
pub struct MenuManager {
    file_menu: gio::Menu,
    edit_menu: gio::Menu,
    text_menu: gio::Menu,
    document_menu: gio::Menu,
    layer_menu: gio::Menu,
    select_menu: gio::Menu,
    arrange_menu: gio::Menu,
    filters_menu: gio::Menu,
    view_menu: gio::Menu,
    window_menu: gio::Menu,
    help_menu: gio::Menu,
    actions: gio::SimpleActionGroup,
    menu_bar: gio::MenuModel,
    callbacks: RefCell<Vec<Box<dyn Fn(&gtk4::Window) + 'static>>>,
}

impl MenuManager {
    pub fn new() -> Self {
        let file_menu = gio::Menu::new();
        let edit_menu = gio::Menu::new();
        let text_menu = gio::Menu::new();
        let document_menu = gio::Menu::new();
        let layer_menu = gio::Menu::new();
        let select_menu = gio::Menu::new();
        let arrange_menu = gio::Menu::new();
        let filters_menu = gio::Menu::new();
        let view_menu = gio::Menu::new();
        let window_menu = gio::Menu::new();
        let help_menu = gio::Menu::new();
        let actions = gio::SimpleActionGroup::new();

        // Create the menu bar
        let menu_bar = gio::Menu::new();
        
        // Create submenus
        menu_bar.append_submenu(Some("File"), &file_menu);
        menu_bar.append_submenu(Some("Edit"), &edit_menu);
        menu_bar.append_submenu(Some("Text"), &text_menu);
        menu_bar.append_submenu(Some("Document"), &document_menu);
        menu_bar.append_submenu(Some("Layer"), &layer_menu);
        menu_bar.append_submenu(Some("Select"), &select_menu);
        menu_bar.append_submenu(Some("Arrange"), &arrange_menu);
        menu_bar.append_submenu(Some("Filters"), &filters_menu);
        menu_bar.append_submenu(Some("View"), &view_menu);
        menu_bar.append_submenu(Some("Window"), &window_menu);
        menu_bar.append_submenu(Some("Help"), &help_menu);

        let mut menu_manager = Self {
            file_menu,
            edit_menu,
            text_menu,
            document_menu,
            layer_menu,
            select_menu,
            arrange_menu,
            filters_menu,
            view_menu,
            window_menu,
            help_menu,
            actions,
            menu_bar: menu_bar.upcast(),
            callbacks: RefCell::new(Vec::new()),
        };

        menu_manager.build_file_menu();
        menu_manager.build_edit_menu();
        menu_manager.build_text_menu();
        menu_manager.build_document_menu();
        menu_manager.build_layer_menu();
        menu_manager.build_select_menu();
        menu_manager.build_arrange_menu();
        menu_manager.build_filters_menu();
        menu_manager.build_view_menu();
        menu_manager.build_window_menu();
        menu_manager.build_help_menu();

        menu_manager
    }

    pub fn get_menu_bar(&self) -> gio::MenuModel {
        self.menu_bar.clone()
    }

    pub fn get_actions(&self) -> &gio::SimpleActionGroup {
        &self.actions
    }

    pub fn add_callback<F>(&self, callback: F)
    where
        F: Fn(&gtk4::Window) + 'static,
    {
        self.callbacks.borrow_mut().push(Box::new(callback));
    }

    pub fn trigger_callback(&self, index: usize) {
        if let Some(callback) = self.callbacks.borrow().get(index) {
            callback(&gtk4::Window::new());
        }
    }

    fn build_file_menu(&mut self) {
        // File menu section - New
        let new_section = gio::Menu::new();
        new_section.append(Some("New"), Some("app.new"));
        new_section.append(Some("New from Clipboard"), Some("app.new_from_clipboard"));
        self.file_menu.append_section(None, &new_section);

        // File menu section - Open
        let open_section = gio::Menu::new();
        open_section.append(Some("Open..."), Some("app.open"));
        open_section.append(Some("Open Recent"), Some("app.open_recent"));
        self.file_menu.append_section(None, &open_section);

        // File menu section - Save
        let save_section = gio::Menu::new();
        save_section.append(Some("Save"), Some("app.save"));
        save_section.append(Some("Save As..."), Some("app.save_as"));
        self.file_menu.append_section(None, &save_section);

        // File menu section - Export
        let export_section = gio::Menu::new();
        export_section.append(Some("Export..."), Some("app.export"));
        export_section.append(Some("Export As..."), Some("app.export_as"));
        self.file_menu.append_section(None, &export_section);

        // File menu section - End
        let end_section = gio::Menu::new();
        end_section.append(Some("Close"), Some("app.close"));
        end_section.append(Some("Quit"), Some("app.quit"));
        self.file_menu.append_section(None, &end_section);

        // Add actions
        self.add_simple_action("new", |_| {
            println!("New document");
        });
        
        self.add_simple_action("open", |_| {
            println!("Open document");
        });
        
        self.add_simple_action("save", |_| {
            println!("Save document");
        });
    }

    fn build_edit_menu(&mut self) {
        // Edit - Undo/Redo
        let undo_section = gio::Menu::new();
        undo_section.append(Some("Undo"), Some("app.undo"));
        undo_section.append(Some("Redo"), Some("app.redo"));
        self.edit_menu.append_section(None, &undo_section);

        // Edit - Clipboard
        let clipboard_section = gio::Menu::new();
        clipboard_section.append(Some("Cut"), Some("app.cut"));
        clipboard_section.append(Some("Copy"), Some("app.copy"));
        clipboard_section.append(Some("Paste"), Some("app.paste"));
        clipboard_section.append(Some("Delete"), Some("app.delete"));
        self.edit_menu.append_section(None, &clipboard_section);

        // Edit - Preferences
        let preferences_section = gio::Menu::new();
        preferences_section.append(Some("Preferences..."), Some("app.preferences"));
        self.edit_menu.append_section(None, &preferences_section);
        
        // Add actions
        self.add_simple_action("undo", |_| {
            println!("Undo");
        });
        
        self.add_simple_action("redo", |_| {
            println!("Redo");
        });
        
        self.add_simple_action("preferences", |window| {
            println!("Opening preferences dialog");
        });
    }

    fn build_text_menu(&mut self) {
        // Basic text operations
        let text_section = gio::Menu::new();
        text_section.append(Some("Bold"), Some("app.text_bold"));
        text_section.append(Some("Italic"), Some("app.text_italic"));
        text_section.append(Some("Underline"), Some("app.text_underline"));
        self.text_menu.append_section(None, &text_section);

        // Text alignment
        let alignment_section = gio::Menu::new();
        alignment_section.append(Some("Align Left"), Some("app.text_align_left"));
        alignment_section.append(Some("Align Center"), Some("app.text_align_center"));
        alignment_section.append(Some("Align Right"), Some("app.text_align_right"));
        alignment_section.append(Some("Justify"), Some("app.text_justify"));
        self.text_menu.append_section(None, &alignment_section);
    }

    fn build_document_menu(&mut self) {
        // Document manipulations
        let doc_section = gio::Menu::new();
        doc_section.append(Some("Resize..."), Some("app.document_resize"));
        doc_section.append(Some("Rotate Canvas..."), Some("app.document_rotate"));
        self.document_menu.append_section(None, &doc_section);

        // Color profiles
        let color_section = gio::Menu::new();
        color_section.append(Some("Convert to sRGB"), Some("app.document_convert_srgb"));
        color_section.append(Some("Convert to AdobeRGB"), Some("app.document_convert_adobergb"));
        self.document_menu.append_section(None, &color_section);

        // Color depth
        let depth_section = gio::Menu::new();
        depth_section.append(Some("Convert to 8-bit"), Some("app.document_convert_8bit"));
        depth_section.append(Some("Convert to 16-bit"), Some("app.document_convert_16bit"));
        depth_section.append(Some("Convert to 32-bit"), Some("app.document_convert_32bit"));
        self.document_menu.append_section(None, &depth_section);
    }

    fn build_layer_menu(&mut self) {
        // Layer creation
        let create_section = gio::Menu::new();
        create_section.append(Some("New Layer"), Some("app.new_layer"));
        create_section.append(Some("New Layer from Selection"), Some("app.new_layer_selection"));
        create_section.append(Some("New Group"), Some("app.new_group"));
        create_section.append(Some("New Fill Layer"), Some("app.new_fill_layer"));
        
        // Adjustment layers submenu
        let adj_layer_submenu = gio::Menu::new();
        adj_layer_submenu.append(Some("New Adjustment Layer"), Some("app.new_adjustment_layer"));
        create_section.append_submenu(None, &adj_layer_submenu);
        
        // Live filter submenu
        let live_filter_submenu = gio::Menu::new();
        live_filter_submenu.append(Some("New Live Filter Layer"), Some("app.new_live_filter"));
        create_section.append_submenu(None, &live_filter_submenu);
        
        self.layer_menu.append_section(None, &create_section);

        // Layer operations
        let ops_section = gio::Menu::new();
        ops_section.append(Some("Delete"), Some("app.delete_layer"));
        ops_section.append(Some("Duplicate"), Some("app.duplicate_layer"));
        ops_section.append(Some("Duplicate Linked"), Some("app.duplicate_linked"));
        self.layer_menu.append_section(None, &ops_section);

        // Layer visibility
        let visibility_section = gio::Menu::new();
        visibility_section.append(Some("Lock"), Some("app.lock_layer"));
        visibility_section.append(Some("Hide"), Some("app.hide_layer"));
        visibility_section.append(Some("Show All"), Some("app.show_all_layers"));
        self.layer_menu.append_section(None, &visibility_section);

        // Layer merging
        let merge_section = gio::Menu::new();
        merge_section.append(Some("Merge Down"), Some("app.merge_down"));
        merge_section.append(Some("Merge Visible"), Some("app.merge_visible"));
        self.layer_menu.append_section(None, &merge_section);

        // Layer rasterization
        let raster_section = gio::Menu::new();
        raster_section.append(Some("Rasterize"), Some("app.rasterize_layer"));
        raster_section.append(Some("Rasterize to Mask"), Some("app.rasterize_to_mask"));
        
        // Geometry submenu
        let geometry_submenu = gio::Menu::new();
        geometry_submenu.append(Some("Convert to Curves"), Some("app.convert_to_curves"));
        geometry_submenu.append(Some("Convert to Text Path"), Some("app.convert_to_text_path"));
        geometry_submenu.append(Some("Convert to Pixels"), Some("app.convert_to_pixels"));
        raster_section.append_submenu(Some("Geometry"), &geometry_submenu);
        
        self.layer_menu.append_section(None, &raster_section);
    }

    fn build_select_menu(&mut self) {
        // Selection basics
        let basics_section = gio::Menu::new();
        basics_section.append(Some("Select All"), Some("app.select_all"));
        basics_section.append(Some("Deselect"), Some("app.deselect"));
        basics_section.append(Some("Reselect"), Some("app.reselect"));
        basics_section.append(Some("Invert Pixel Selection"), Some("app.invert_selection"));
        self.select_menu.append_section(None, &basics_section);

        // Selection modifications
        let modify_section = gio::Menu::new();
        modify_section.append(Some("Selection from Layer Visibility"), Some("app.selection_from_layer"));
        modify_section.append(Some("Selection from Layer"), Some("app.selection_from_layer_contents"));
        modify_section.append(Some("Selection from Layer and Outline"), Some("app.selection_from_layer_and_outline"));
        self.select_menu.append_section(None, &modify_section);
        
        // Selection operations
        let ops_section = gio::Menu::new();
        ops_section.append(Some("Save Selection"), Some("app.save_selection"));
        ops_section.append(Some("Load Selection from File..."), Some("app.load_selection"));
        self.select_menu.append_section(None, &ops_section);
        
        // Selection box operations
        let box_section = gio::Menu::new();
        box_section.append(Some("Cycle Selection Box"), Some("app.cycle_selection_box"));
        box_section.append(Some("Set Selection Box..."), Some("app.set_selection_box"));
        self.select_menu.append_section(None, &box_section);
        
        // Advanced selection operations
        let advanced_section = gio::Menu::new();
        advanced_section.append(Some("Select All Layers"), Some("app.select_all_layers"));
        advanced_section.append(Some("Deselect Layers"), Some("app.deselect_layers"));
        advanced_section.append(Some("Select Parent Layer"), Some("app.select_parent_layer"));
        advanced_section.append(Some("Select Previous Layer"), Some("app.select_previous_layer"));
        advanced_section.append(Some("Select Top Layer"), Some("app.select_top_layer"));
        advanced_section.append(Some("Select Bottom Layer"), Some("app.select_bottom_layer"));
        self.select_menu.append_section(None, &advanced_section);
        
        // Color range selections
        let color_range_section = gio::Menu::new();
        color_range_section.append(Some("Select Subject"), Some("app.select_subject"));
        
        // Color range submenu
        let color_range_submenu = gio::Menu::new();
        color_range_submenu.append(Some("Color Range..."), Some("app.color_range_selection"));
        color_range_section.append_submenu(None, &color_range_submenu);
        
        // Alpha range submenu
        let alpha_range_submenu = gio::Menu::new();
        alpha_range_submenu.append(Some("Alpha Range..."), Some("app.alpha_range_selection"));
        color_range_section.append_submenu(None, &alpha_range_submenu);
        
        self.select_menu.append_section(None, &color_range_section);
        
        // Selection finalization
        let final_section = gio::Menu::new();
        final_section.append(Some("Select Sampled Color"), Some("app.select_sampled_color"));
        final_section.append(Some("Grow / Shrink..."), Some("app.grow_shrink_selection"));
        final_section.append(Some("Feather..."), Some("app.feather_selection"));
        final_section.append(Some("Smooth..."), Some("app.smooth_selection"));
        final_section.append(Some("Border..."), Some("app.border_selection"));
        final_section.append(Some("Refine Edges..."), Some("app.refine_selection_edges"));
        final_section.append(Some("Edit Selection as Layer"), Some("app.edit_selection_as_layer"));
        self.select_menu.append_section(None, &final_section);
    }

    fn build_arrange_menu(&mut self) {
        // Basic arrangements
        let basics_section = gio::Menu::new();
        basics_section.append(Some("Use Stack Groups"), Some("app.use_stack_groups"));
        basics_section.append(Some("Organize..."), Some("app.organize"));
        self.arrange_menu.append_section(None, &basics_section);
        
        // Move operations
        let move_section = gio::Menu::new();
        move_section.append(Some("Move to Front"), Some("app.move_to_front"));
        move_section.append(Some("Move Forward One"), Some("app.move_forward_one"));
        move_section.append(Some("Move Back One"), Some("app.move_back_one"));
        move_section.append(Some("Move to Back"), Some("app.move_to_back"));
        self.arrange_menu.append_section(None, &move_section);
        
        // Alignment
        let align_section = gio::Menu::new();
        align_section.append(Some("Align Left"), Some("app.align_left"));
        align_section.append(Some("Align Center"), Some("app.align_center"));
        align_section.append(Some("Align Right"), Some("app.align_right"));
        align_section.append(Some("Align Top"), Some("app.align_top"));
        align_section.append(Some("Align Middle"), Some("app.align_middle"));
        align_section.append(Some("Align Bottom"), Some("app.align_bottom"));
        align_section.append(Some("Align Layers by Stack"), Some("app.align_layers_by_stack"));
        self.arrange_menu.append_section(None, &align_section);
        
        // Flip operations
        let flip_section = gio::Menu::new();
        flip_section.append(Some("Flip Horizontal"), Some("app.flip_horizontal"));
        flip_section.append(Some("Flip Vertical"), Some("app.flip_vertical"));
        flip_section.append(Some("Rotate 90° Clockwise"), Some("app.rotate_90_clockwise"));
        flip_section.append(Some("Rotate 90° Counterclockwise"), Some("app.rotate_90_counterclockwise"));
        self.arrange_menu.append_section(None, &flip_section);
    }

    fn build_filters_menu(&mut self) {
        // Filter categories
        let sharpen_section = gio::Menu::new();
        let sharpen_submenu = gio::Menu::new();
        sharpen_submenu.append(Some("Sharpen"), Some("app.filter_sharpen"));
        sharpen_submenu.append(Some("Gaussian Blur..."), Some("app.filter_gaussian_blur"));
        sharpen_submenu.append(Some("Perspective Blur..."), Some("app.filter_perspective_blur"));
        sharpen_submenu.append(Some("Tilt / Shift..."), Some("app.filter_tilt_shift"));
        sharpen_submenu.append(Some("Pen / Pencil..."), Some("app.filter_pen_pencil"));
        sharpen_submenu.append(Some("Unsharp..."), Some("app.filter_unsharp"));
        sharpen_section.append_submenu(Some("Sharpen"), &sharpen_submenu);
        self.filters_menu.append_section(None, &sharpen_section);
        
        // Blur filters
        let blur_section = gio::Menu::new();
        let blur_submenu = gio::Menu::new();
        blur_submenu.append(Some("Average Blur..."), Some("app.filter_average_blur"));
        blur_submenu.append(Some("Gaussian Blur..."), Some("app.filter_gaussian_blur"));
        blur_submenu.append(Some("Box Blur..."), Some("app.filter_box_blur"));
        blur_submenu.append(Some("Median Blur..."), Some("app.filter_median_blur"));
        blur_submenu.append(Some("Depth of Field Blur..."), Some("app.filter_depth_of_field"));
        blur_submenu.append(Some("Field Blur..."), Some("app.filter_field_blur"));
        blur_submenu.append(Some("Premium Blur..."), Some("app.filter_premium_blur"));
        blur_submenu.append(Some("Maximum Blur..."), Some("app.filter_maximum_blur"));
        blur_submenu.append(Some("Radial Blur..."), Some("app.filter_radial_blur"));
        blur_submenu.append(Some("Orbital Blur..."), Some("app.filter_orbital_blur"));
        blur_submenu.append(Some("Custom Blur..."), Some("app.filter_custom_blur"));
        blur_section.append_submenu(Some("Blur"), &blur_submenu);
        self.filters_menu.append_section(None, &blur_section);
        
        // Distortion filters
        let distort_section = gio::Menu::new();
        let distort_submenu = gio::Menu::new();
        distort_submenu.append(Some("Clarity..."), Some("app.filter_clarity"));
        distort_submenu.append(Some("High Pass..."), Some("app.filter_high_pass"));
        distort_submenu.append(Some("Defocus..."), Some("app.filter_defocus"));
        distort_submenu.append(Some("Refraction..."), Some("app.filter_refraction"));
        distort_section.append_submenu(Some("Distort"), &distort_submenu);
        self.filters_menu.append_section(None, &distort_section);
        
        // Frequency Separation
        let freq_section = gio::Menu::new();
        freq_section.append(Some("Frequency Separation..."), Some("app.filter_frequency_separation"));
        self.filters_menu.append_section(None, &freq_section);
        
        // Color adjustments
        let color_section = gio::Menu::new();
        color_section.append(Some("Apply Brush..."), Some("app.filter_apply_brush"));
        color_section.append(Some("Lighting..."), Some("app.filter_lighting"));
        color_section.append(Some("Shadows & Highlights..."), Some("app.filter_shadows_highlights"));
        color_section.append(Some("Haze Removal..."), Some("app.filter_haze_removal"));
        self.filters_menu.append_section(None, &color_section);
        
        // Plugins
        let plugin_section = gio::Menu::new();
        plugin_section.append(Some("Plugins..."), Some("app.plugins"));
        self.filters_menu.append_section(None, &plugin_section);
    }

    fn build_view_menu(&mut self) {
        // View zoom options
        let zoom_section = gio::Menu::new();
        zoom_section.append(Some("Zoom In"), Some("app.zoom_in"));
        zoom_section.append(Some("Zoom Out"), Some("app.zoom_out"));
        zoom_section.append(Some("Zoom to 100%"), Some("app.zoom_100"));
        zoom_section.append(Some("Zoom to Fit"), Some("app.zoom_fit"));
        self.view_menu.append_section(None, &zoom_section);
        
        // View panels
        let panel_section = gio::Menu::new();
        panel_section.append(Some("Show Tools Panel"), Some("app.show_tools_panel"));
        panel_section.append(Some("Show Layers Panel"), Some("app.show_layers_panel"));
        panel_section.append(Some("Show Histogram"), Some("app.show_histogram"));
        panel_section.append(Some("Show Navigator"), Some("app.show_navigator"));
        self.view_menu.append_section(None, &panel_section);
        
        // View modes
        let mode_section = gio::Menu::new();
        mode_section.append(Some("Full Screen"), Some("app.full_screen"));
        mode_section.append(Some("Presentation Mode"), Some("app.presentation_mode"));
        mode_section.append(Some("Split View"), Some("app.split_view"));
        self.view_menu.append_section(None, &mode_section);
    }

    fn build_window_menu(&mut self) {
        // Window management
        let window_section = gio::Menu::new();
        window_section.append(Some("New Window"), Some("app.new_window"));
        window_section.append(Some("Close Window"), Some("app.close_window"));
        self.window_menu.append_section(None, &window_section);
        
        // Layout options
        let layout_section = gio::Menu::new();
        layout_section.append(Some("Reset Layout"), Some("app.reset_layout"));
        layout_section.append(Some("Save Layout..."), Some("app.save_layout"));
        layout_section.append(Some("Load Layout..."), Some("app.load_layout"));
        self.window_menu.append_section(None, &layout_section);
        
        // Workspace options
        let workspace_section = gio::Menu::new();
        workspace_section.append(Some("Photo Workspace"), Some("app.photo_workspace"));
        workspace_section.append(Some("Liquify Workspace"), Some("app.liquify_workspace"));
        workspace_section.append(Some("Develop Workspace"), Some("app.develop_workspace"));
        workspace_section.append(Some("Tone Mapping Workspace"), Some("app.tone_mapping_workspace"));
        self.window_menu.append_section(None, &workspace_section);
    }

    fn build_help_menu(&mut self) {
        // Help resources
        let help_section = gio::Menu::new();
        help_section.append(Some("Documentation"), Some("app.documentation"));
        help_section.append(Some("Video Tutorials"), Some("app.video_tutorials"));
        help_section.append(Some("Forum"), Some("app.forum"));
        self.help_menu.append_section(None, &help_section);
        
        // About
        let about_section = gio::Menu::new();
        about_section.append(Some("Check for Updates"), Some("app.check_updates"));
        about_section.append(Some("About"), Some("app.about"));
        self.help_menu.append_section(None, &about_section);
    }

    fn add_simple_action<F>(&mut self, name: &str, callback: F)
    where
        F: Fn(&gtk4::Window) + 'static,
    {
        let action = gio::SimpleAction::new(name, None);
        let callback = Rc::new(RefCell::new(callback));
        
        action.connect_activate(move |_, _| {
            // In a real app, we'd pass the actual window
            callback.borrow()(&gtk4::Window::new());
        });
        
        self.actions.add_action(&action);
    }
} 