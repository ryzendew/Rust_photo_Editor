use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CellRendererText, ComboBoxText, Dialog, DialogFlags,
    Entry, Grid, HeaderBar, Label, ListStore, Notebook, ResponseType, Scale, SpinButton,
    Switch, TreeView, TreeViewColumn,
};
use libadwaita as adw;
use adw::prelude::*;
use adw::{HeaderBar as AdwHeaderBar, Clamp, WindowTitle};
use std::rc::Rc;
use std::cell::RefCell;
use log::{debug, info};

use crate::core::settings::{Settings, PerformanceSettings, SaveSettings, DisplaySettings};

pub struct SettingsDialog {
    pub dialog: Dialog,
    pub settings: Rc<RefCell<Settings>>,
}

impl SettingsDialog {
    pub fn new(parent: &impl IsA<gtk4::Window>) -> Self {
        info!("Creating settings dialog");
        
        // Load settings (or create default)
        let settings = Settings::load().unwrap_or_default();
        let settings = Rc::new(RefCell::new(settings));
        
        // Create the dialog
        let dialog = Dialog::new();
        dialog.set_title(Some("Settings"));
        dialog.set_transient_for(Some(parent));
        dialog.set_modal(true);
        dialog.set_default_width(500);
        dialog.set_default_height(400);
        
        // Create the header bar
        let header_bar = HeaderBar::new();
        dialog.set_titlebar(Some(&header_bar));
        
        // Create the content area
        let content_area = dialog.content_area();
        content_area.set_margin_start(10);
        content_area.set_margin_end(10);
        content_area.set_margin_top(10);
        content_area.set_margin_bottom(10);
        
        // Create a notebook with tabs for different setting categories
        let notebook = Notebook::new();
        content_area.append(&notebook);
        
        // Create General settings tab
        let general_page = SettingsDialog::create_general_tab(&settings);
        notebook.append_page(&general_page, Some(&Label::new(Some("General"))));
        
        // Create Performance settings tab
        let performance_page = SettingsDialog::create_performance_tab(&settings);
        notebook.append_page(&performance_page, Some(&Label::new(Some("Performance"))));
        
        // Create Save settings tab
        let save_page = SettingsDialog::create_save_tab(&settings);
        notebook.append_page(&save_page, Some(&Label::new(Some("Save"))));
        
        // Create Display settings tab
        let display_page = SettingsDialog::create_display_tab(&settings);
        notebook.append_page(&display_page, Some(&Label::new(Some("Display"))));
        
        // Add buttons
        dialog.add_button("Cancel", ResponseType::Cancel);
        dialog.add_button("OK", ResponseType::Ok);
        
        // Create the dialog instance
        let dialog_instance = SettingsDialog {
            dialog,
            settings,
        };
        
        // Connect the response signal
        let settings_clone = dialog_instance.settings.clone();
        dialog_instance.dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Ok {
                info!("Saving settings");
                let settings = settings_clone.borrow();
                if let Err(e) = settings.save() {
                    eprintln!("Failed to save settings: {}", e);
                }
            }
            dialog.destroy();
        });
        
        info!("Settings dialog created");
        dialog_instance
    }
    
    pub fn show(&self) {
        self.dialog.show();
    }
    
    fn create_general_tab(settings: &Rc<RefCell<Settings>>) -> GtkBox {
        let page = GtkBox::new(gtk4::Orientation::Vertical, 10);
        page.set_margin_start(10);
        page.set_margin_end(10);
        page.set_margin_top(10);
        page.set_margin_bottom(10);
        
        // Create a grid for layout
        let grid = Grid::new();
        grid.set_row_spacing(10);
        grid.set_column_spacing(10);
        grid.set_margin_top(10);
        grid.set_margin_bottom(10);
        page.append(&grid);
        
        // Theme setting
        let theme_label = Label::new(Some("Theme:"));
        theme_label.set_halign(Align::Start);
        grid.attach(&theme_label, 0, 0, 1, 1);
        
        let theme_combo = ComboBoxText::new();
        theme_combo.append(Some("light"), "Light");
        theme_combo.append(Some("dark"), "Dark");
        theme_combo.append(Some("system"), "System Default");
        theme_combo.set_active_id(Some(settings.borrow().theme.as_str()));
        grid.attach(&theme_combo, 1, 0, 1, 1);
        
        // Language setting
        let lang_label = Label::new(Some("Language:"));
        lang_label.set_halign(Align::Start);
        grid.attach(&lang_label, 0, 1, 1, 1);
        
        let lang_combo = ComboBoxText::new();
        lang_combo.append(Some("en-US"), "English (US)");
        lang_combo.append(Some("en-GB"), "English (UK)");
        lang_combo.append(Some("fr-FR"), "French");
        lang_combo.append(Some("de-DE"), "German");
        lang_combo.append(Some("es-ES"), "Spanish");
        lang_combo.append(Some("it-IT"), "Italian");
        lang_combo.append(Some("ja-JP"), "Japanese");
        lang_combo.append(Some("zh-CN"), "Chinese (Simplified)");
        lang_combo.set_active_id(Some(settings.borrow().language.as_str()));
        grid.attach(&lang_combo, 1, 1, 1, 1);
        
        // Auto-save setting
        let autosave_label = Label::new(Some("Auto-save:"));
        autosave_label.set_halign(Align::Start);
        grid.attach(&autosave_label, 0, 2, 1, 1);
        
        let autosave_switch = Switch::new();
        autosave_switch.set_active(settings.borrow().auto_save);
        autosave_switch.set_halign(Align::Start);
        grid.attach(&autosave_switch, 1, 2, 1, 1);
        
        // Auto-save interval setting
        let interval_label = Label::new(Some("Auto-save interval (minutes):"));
        interval_label.set_halign(Align::Start);
        grid.attach(&interval_label, 0, 3, 1, 1);
        
        let interval_spin = SpinButton::with_range(1.0, 60.0, 1.0);
        interval_spin.set_value(settings.borrow().auto_save_interval as f64);
        grid.attach(&interval_spin, 1, 3, 1, 1);
        
        // Connect signals to update settings
        let settings_clone = settings.clone();
        theme_combo.connect_changed(move |combo| {
            if let Some(theme) = combo.active_id() {
                settings_clone.borrow_mut().theme = theme.to_string();
            }
        });
        
        let settings_clone = settings.clone();
        lang_combo.connect_changed(move |combo| {
            if let Some(lang) = combo.active_id() {
                settings_clone.borrow_mut().language = lang.to_string();
            }
        });
        
        let settings_clone = settings.clone();
        autosave_switch.connect_state_set(move |_, state| {
            settings_clone.borrow_mut().auto_save = state;
            false.into()
        });
        
        let settings_clone = settings.clone();
        interval_spin.connect_value_changed(move |spin| {
            settings_clone.borrow_mut().auto_save_interval = spin.value() as u32;
        });
        
        page
    }
    
    fn create_performance_tab(settings: &Rc<RefCell<Settings>>) -> GtkBox {
        let page = GtkBox::new(gtk4::Orientation::Vertical, 10);
        page.set_margin_start(10);
        page.set_margin_end(10);
        page.set_margin_top(10);
        page.set_margin_bottom(10);
        
        // Create a grid for layout
        let grid = Grid::new();
        grid.set_row_spacing(10);
        grid.set_column_spacing(10);
        grid.set_margin_top(10);
        grid.set_margin_bottom(10);
        page.append(&grid);
        
        // Memory usage setting
        let memory_label = Label::new(Some("Memory Usage (%):"));
        memory_label.set_halign(Align::Start);
        grid.attach(&memory_label, 0, 0, 1, 1);
        
        let memory_scale = Scale::with_range(gtk4::Orientation::Horizontal, 10.0, 90.0, 5.0);
        memory_scale.set_value(settings.borrow().performance.memory_usage_percent as f64);
        memory_scale.set_hexpand(true);
        grid.attach(&memory_scale, 1, 0, 1, 1);
        
        // Undo levels setting
        let undo_label = Label::new(Some("Undo Levels:"));
        undo_label.set_halign(Align::Start);
        grid.attach(&undo_label, 0, 1, 1, 1);
        
        let undo_spin = SpinButton::with_range(1.0, 100.0, 1.0);
        undo_spin.set_value(settings.borrow().performance.undo_levels as f64);
        grid.attach(&undo_spin, 1, 1, 1, 1);
        
        // Use GPU acceleration setting
        let gpu_label = Label::new(Some("Use GPU Acceleration:"));
        gpu_label.set_halign(Align::Start);
        grid.attach(&gpu_label, 0, 2, 1, 1);
        
        let gpu_switch = Switch::new();
        gpu_switch.set_active(settings.borrow().performance.use_gpu);
        gpu_switch.set_halign(Align::Start);
        grid.attach(&gpu_switch, 1, 2, 1, 1);
        
        // Thread count setting
        let thread_label = Label::new(Some("Thread Count:"));
        thread_label.set_halign(Align::Start);
        grid.attach(&thread_label, 0, 3, 1, 1);
        
        let thread_spin = SpinButton::with_range(1.0, 32.0, 1.0);
        thread_spin.set_value(settings.borrow().performance.thread_count as f64);
        grid.attach(&thread_spin, 1, 3, 1, 1);
        
        // Connect signals to update settings
        let settings_clone = settings.clone();
        memory_scale.connect_value_changed(move |scale| {
            settings_clone.borrow_mut().performance.memory_usage_percent = scale.value() as u32;
        });
        
        let settings_clone = settings.clone();
        undo_spin.connect_value_changed(move |spin| {
            settings_clone.borrow_mut().performance.undo_levels = spin.value() as u32;
        });
        
        let settings_clone = settings.clone();
        gpu_switch.connect_state_set(move |_, state| {
            settings_clone.borrow_mut().performance.use_gpu = state;
            false.into()
        });
        
        let settings_clone = settings.clone();
        thread_spin.connect_value_changed(move |spin| {
            settings_clone.borrow_mut().performance.thread_count = spin.value() as u32;
        });
        
        page
    }
    
    fn create_save_tab(settings: &Rc<RefCell<Settings>>) -> GtkBox {
        let page = GtkBox::new(gtk4::Orientation::Vertical, 10);
        page.set_margin_start(10);
        page.set_margin_end(10);
        page.set_margin_top(10);
        page.set_margin_bottom(10);
        
        // Create a grid for layout
        let grid = Grid::new();
        grid.set_row_spacing(10);
        grid.set_column_spacing(10);
        grid.set_margin_top(10);
        grid.set_margin_bottom(10);
        page.append(&grid);
        
        // Default format setting
        let format_label = Label::new(Some("Default Format:"));
        format_label.set_halign(Align::Start);
        grid.attach(&format_label, 0, 0, 1, 1);
        
        let format_combo = ComboBoxText::new();
        format_combo.append(Some("png"), "PNG");
        format_combo.append(Some("jpeg"), "JPEG");
        format_combo.append(Some("tiff"), "TIFF");
        format_combo.append(Some("webp"), "WebP");
        format_combo.append(Some("native"), "Native Format");
        format_combo.set_active_id(Some(settings.borrow().save.default_format.as_str()));
        grid.attach(&format_combo, 1, 0, 1, 1);
        
        // JPEG quality setting
        let jpeg_label = Label::new(Some("JPEG Quality:"));
        jpeg_label.set_halign(Align::Start);
        grid.attach(&jpeg_label, 0, 1, 1, 1);
        
        let jpeg_scale = Scale::with_range(gtk4::Orientation::Horizontal, 1.0, 100.0, 1.0);
        jpeg_scale.set_value(settings.borrow().save.jpeg_quality as f64);
        jpeg_scale.set_hexpand(true);
        grid.attach(&jpeg_scale, 1, 1, 1, 1);
        
        // PNG compression setting
        let png_label = Label::new(Some("PNG Compression:"));
        png_label.set_halign(Align::Start);
        grid.attach(&png_label, 0, 2, 1, 1);
        
        let png_scale = Scale::with_range(gtk4::Orientation::Horizontal, 1.0, 9.0, 1.0);
        png_scale.set_value(settings.borrow().save.png_compression as f64);
        png_scale.set_hexpand(true);
        grid.attach(&png_scale, 1, 2, 1, 1);
        
        // Include metadata setting
        let metadata_label = Label::new(Some("Include Metadata:"));
        metadata_label.set_halign(Align::Start);
        grid.attach(&metadata_label, 0, 3, 1, 1);
        
        let metadata_switch = Switch::new();
        metadata_switch.set_active(settings.borrow().save.include_metadata);
        metadata_switch.set_halign(Align::Start);
        grid.attach(&metadata_switch, 1, 3, 1, 1);
        
        // Default save location
        let location_label = Label::new(Some("Default Save Location:"));
        location_label.set_halign(Align::Start);
        grid.attach(&location_label, 0, 4, 1, 1);
        
        let location_entry = Entry::new();
        location_entry.set_text(&settings.borrow().save.default_location);
        location_entry.set_hexpand(true);
        grid.attach(&location_entry, 1, 4, 1, 1);
        
        let browse_button = Button::with_label("Browse...");
        grid.attach(&browse_button, 2, 4, 1, 1);
        
        // Connect signals to update settings
        let settings_clone = settings.clone();
        format_combo.connect_changed(move |combo| {
            if let Some(format) = combo.active_id() {
                settings_clone.borrow_mut().save.default_format = format.to_string();
            }
        });
        
        let settings_clone = settings.clone();
        jpeg_scale.connect_value_changed(move |scale| {
            settings_clone.borrow_mut().save.jpeg_quality = scale.value() as u32;
        });
        
        let settings_clone = settings.clone();
        png_scale.connect_value_changed(move |scale| {
            settings_clone.borrow_mut().save.png_compression = scale.value() as u32;
        });
        
        let settings_clone = settings.clone();
        metadata_switch.connect_state_set(move |_, state| {
            settings_clone.borrow_mut().save.include_metadata = state;
            false.into()
        });
        
        let settings_clone = settings.clone();
        location_entry.connect_changed(move |entry| {
            settings_clone.borrow_mut().save.default_location = entry.text().to_string();
        });
        
        let _entry_clone = location_entry.clone();
        let browse_button = Button::with_label("Browse...");
        grid.attach(&browse_button, 2, 4, 1, 1);
        
        page
    }
    
    fn create_display_tab(settings: &Rc<RefCell<Settings>>) -> GtkBox {
        let page = GtkBox::new(gtk4::Orientation::Vertical, 10);
        page.set_margin_start(10);
        page.set_margin_end(10);
        page.set_margin_top(10);
        page.set_margin_bottom(10);
        
        // Create a grid for layout
        let grid = Grid::new();
        grid.set_row_spacing(10);
        grid.set_column_spacing(10);
        grid.set_margin_top(10);
        grid.set_margin_bottom(10);
        page.append(&grid);
        
        // Color Space setting
        let color_label = Label::new(Some("Color Space:"));
        color_label.set_halign(Align::Start);
        grid.attach(&color_label, 0, 0, 1, 1);
        
        let color_combo = ComboBoxText::new();
        color_combo.append(Some("srgb"), "sRGB");
        color_combo.append(Some("adobe-rgb"), "Adobe RGB");
        color_combo.append(Some("prophoto-rgb"), "ProPhoto RGB");
        color_combo.append(Some("p3"), "Display P3");
        color_combo.set_active_id(Some(settings.borrow().display.color_space.as_str()));
        grid.attach(&color_combo, 1, 0, 1, 1);
        
        // Color Depth setting
        let depth_label = Label::new(Some("Color Depth:"));
        depth_label.set_halign(Align::Start);
        grid.attach(&depth_label, 0, 1, 1, 1);
        
        let depth_combo = ComboBoxText::new();
        depth_combo.append(Some("8"), "8-bit");
        depth_combo.append(Some("16"), "16-bit");
        depth_combo.append(Some("32"), "32-bit");
        depth_combo.set_active_id(Some(&settings.borrow().display.color_depth.to_string()));
        grid.attach(&depth_combo, 1, 1, 1, 1);
        
        // Checkerboard background setting
        let checker_label = Label::new(Some("Checkerboard for Transparency:"));
        checker_label.set_halign(Align::Start);
        grid.attach(&checker_label, 0, 2, 1, 1);
        
        let checker_switch = Switch::new();
        checker_switch.set_active(settings.borrow().display.use_checkerboard);
        checker_switch.set_halign(Align::Start);
        grid.attach(&checker_switch, 1, 2, 1, 1);
        
        // Checkerboard color 1
        let color1_label = Label::new(Some("Checkerboard Color 1:"));
        color1_label.set_halign(Align::Start);
        grid.attach(&color1_label, 0, 3, 1, 1);
        
        let color1_button = Button::new();
        color1_button.set_label(&settings.borrow().display.checkerboard_color1);
        grid.attach(&color1_button, 1, 3, 1, 1);
        
        // Checkerboard color 2
        let color2_label = Label::new(Some("Checkerboard Color 2:"));
        color2_label.set_halign(Align::Start);
        grid.attach(&color2_label, 0, 4, 1, 1);
        
        let color2_button = Button::new();
        color2_button.set_label(&settings.borrow().display.checkerboard_color2);
        grid.attach(&color2_button, 1, 4, 1, 1);
        
        // Connect signals to update settings
        let settings_clone = settings.clone();
        color_combo.connect_changed(move |combo| {
            if let Some(color) = combo.active_id() {
                settings_clone.borrow_mut().display.color_space = color.to_string();
            }
        });
        
        let settings_clone = settings.clone();
        depth_combo.connect_changed(move |combo| {
            if let Some(depth) = combo.active_id() {
                if let Ok(depth_val) = depth.parse::<u8>() {
                    settings_clone.borrow_mut().display.color_depth = depth_val;
                }
            }
        });
        
        let settings_clone = settings.clone();
        checker_switch.connect_state_set(move |_, state| {
            settings_clone.borrow_mut().display.use_checkerboard = state;
            false.into()
        });
        
        color1_button.connect_clicked(|_btn| {
            // Here you would show a color chooser dialog
            // and update the button label and setting with the selected color
        });
        
        color2_button.connect_clicked(|_btn| {
            // Here you would show a color chooser dialog
            // and update the button label and setting with the selected color
        });
        
        page
    }
} 