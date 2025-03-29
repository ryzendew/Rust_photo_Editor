use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CellRendererText, ComboBoxText, Dialog, DialogFlags,
    Entry, Grid, HeaderBar, Label, ListStore, Notebook, Orientation, ResponseType, Scale, SpinButton,
    Switch, TreeView, TreeViewColumn, ApplicationWindow, Adjustment,
};
use gtk4::glib::clone;
use libadwaita as adw;
use adw::prelude::*;
use adw::{HeaderBar as AdwHeaderBar, Clamp, WindowTitle};
use std::rc::Rc;
use std::cell::RefCell;
use log::{debug, info, error};

use crate::core::settings::{Settings, PerformanceSettings, SaveSettings, DisplaySettings};

pub struct PreferencesDialog {
    dialog: Dialog,
    settings: Rc<RefCell<Settings>>,
}

impl PreferencesDialog {
    pub fn new(parent: &ApplicationWindow) -> Self {
        let dialog = Dialog::with_buttons(
            Some("Preferences"),
            Some(parent),
            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT | DialogFlags::USE_HEADER_BAR,
            &[("Cancel", ResponseType::Cancel), ("OK", ResponseType::Ok)]
        );

        let settings = Settings::load().expect("Failed to load settings");
        let settings = Rc::new(RefCell::new(settings));

        let mut dialog_obj = Self {
            dialog,
            settings,
        };

        dialog_obj.build_ui();
        dialog_obj
    }

    pub fn show(&self) {
        self.dialog.show();
    }

    fn build_ui(&mut self) {
        let content_area = self.dialog.content_area();
        let notebook = Notebook::new();
        content_area.append(&notebook);

        // Add tabs
        notebook.append_page(
            &self.create_performance_tab(),
            Some(&Label::new(Some("Performance"))),
        );
        notebook.append_page(
            &self.create_save_tab(),
            Some(&Label::new(Some("Save"))),
        );
        notebook.append_page(
            &self.create_display_tab(),
            Some(&Label::new(Some("Display"))),
        );

        // Create widgets
        let gpu_switch = Switch::new();
        gpu_switch.set_active(self.settings.borrow().performance.use_gpu);
        let settings = self.settings.clone();
        gpu_switch.connect_active_notify(move |switch| {
            let mut settings = settings.borrow_mut();
            settings.performance.use_gpu = switch.is_active();
            info!("GPU acceleration set to {}", switch.is_active());
        });

        let memory_scale = Scale::new(Orientation::Horizontal, Some(&Adjustment::new(50.0, 0.0, 100.0, 1.0, 10.0, 0.0)));
        memory_scale.set_value(self.settings.borrow().performance.memory_usage_percent as f64);
        let settings = self.settings.clone();
        memory_scale.connect_value_changed(move |scale| {
            let mut settings = settings.borrow_mut();
            settings.performance.memory_usage_percent = scale.value() as u32;
            info!("Memory usage set to {}%", scale.value());
        });

        let undo_scale = Scale::new(Orientation::Horizontal, Some(&Adjustment::new(20.0, 1.0, 100.0, 1.0, 10.0, 0.0)));
        undo_scale.set_value(self.settings.borrow().performance.undo_levels as f64);
        let settings = self.settings.clone();
        undo_scale.connect_value_changed(move |scale| {
            let mut settings = settings.borrow_mut();
            settings.performance.undo_levels = scale.value() as u32;
            info!("Undo levels set to {}", scale.value());
        });

        let format_combo = ComboBoxText::new();
        format_combo.append_text("PNG");
        format_combo.append_text("JPEG");
        format_combo.append_text("BMP");
        format_combo.set_active_id(Some(&self.settings.borrow().save.default_format));
        let settings = self.settings.clone();
        format_combo.connect_changed(move |combo| {
            if let Some(format) = combo.active_text() {
                let mut settings = settings.borrow_mut();
                settings.save.default_format = format.to_string();
                info!("Default save format set to {}", format);
            }
        });

        let metadata_switch = Switch::new();
        metadata_switch.set_active(self.settings.borrow().save.include_metadata);
        let settings = self.settings.clone();
        metadata_switch.connect_active_notify(move |switch| {
            let mut settings = settings.borrow_mut();
            settings.save.include_metadata = switch.is_active();
            info!("Include metadata set to {}", switch.is_active());
        });

        let checkerboard_switch = Switch::new();
        checkerboard_switch.set_active(self.settings.borrow().display.use_checkerboard);
        let settings = self.settings.clone();
        checkerboard_switch.connect_active_notify(move |switch| {
            let mut settings = settings.borrow_mut();
            settings.display.use_checkerboard = switch.is_active();
            info!("Use checkerboard set to {}", switch.is_active());
        });

        let colorspace_combo = ComboBoxText::new();
        colorspace_combo.append_text("sRGB");
        colorspace_combo.append_text("Adobe RGB");
        colorspace_combo.append_text("ProPhoto RGB");
        colorspace_combo.set_active_id(Some(&self.settings.borrow().display.color_space));
        let settings = self.settings.clone();
        colorspace_combo.connect_changed(move |combo| {
            if let Some(colorspace) = combo.active_text() {
                let mut settings = settings.borrow_mut();
                settings.display.color_space = colorspace.to_string();
                info!("Display color space set to {}", colorspace);
            }
        });

        // Add buttons
        self.dialog.add_button("Cancel", ResponseType::Cancel);
        self.dialog.add_button("OK", ResponseType::Ok);

        // Connect response signal
        let settings = self.settings.clone();
        self.dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Ok {
                info!("Settings dialog OK clicked");
                if let Err(err) = settings.borrow().save() {
                    error!("Failed to save settings: {}", err);
                }
            }
            dialog.close();
        });

        // Add widgets to containers
        let gpu_box = GtkBox::new(Orientation::Horizontal, 12);
        gpu_box.append(&Label::new(Some("GPU Acceleration")));
        gpu_box.append(&gpu_switch);
        let memory_box = GtkBox::new(Orientation::Horizontal, 12);
        memory_box.append(&Label::new(Some("Memory Usage (%)")));
        memory_box.append(&memory_scale);
        let undo_box = GtkBox::new(Orientation::Horizontal, 12);
        undo_box.append(&Label::new(Some("Undo Levels")));
        undo_box.append(&undo_scale);
        let format_box = GtkBox::new(Orientation::Horizontal, 12);
        format_box.append(&Label::new(Some("Default Format")));
        format_box.append(&format_combo);
        let metadata_box = GtkBox::new(Orientation::Horizontal, 12);
        metadata_box.append(&Label::new(Some("Include Metadata")));
        metadata_box.append(&metadata_switch);
        let checkerboard_box = GtkBox::new(Orientation::Horizontal, 12);
        checkerboard_box.append(&Label::new(Some("Show Transparency Checkerboard")));
        checkerboard_box.append(&checkerboard_switch);
        let colorspace_box = GtkBox::new(Orientation::Horizontal, 12);
        colorspace_box.append(&Label::new(Some("Display Color Space")));
        colorspace_box.append(&colorspace_combo);

        // Add widgets to notebook
        let gpu_tab = GtkBox::new(Orientation::Vertical, 10);
        gpu_tab.set_margin_start(10);
        gpu_tab.set_margin_end(10);
        gpu_tab.set_margin_top(10);
        gpu_tab.set_margin_bottom(10);
        gpu_tab.append(&gpu_box);
        notebook.append_page(
            &gpu_tab,
            Some(&Label::new(Some("Performance"))),
        );

        let save_tab = GtkBox::new(Orientation::Vertical, 10);
        save_tab.set_margin_start(10);
        save_tab.set_margin_end(10);
        save_tab.set_margin_top(10);
        save_tab.set_margin_bottom(10);
        save_tab.append(&format_box);
        save_tab.append(&metadata_box);
        notebook.append_page(
            &save_tab,
            Some(&Label::new(Some("Save"))),
        );

        let display_tab = GtkBox::new(Orientation::Vertical, 10);
        display_tab.set_margin_start(10);
        display_tab.set_margin_end(10);
        display_tab.set_margin_top(10);
        display_tab.set_margin_bottom(10);
        display_tab.append(&checkerboard_box);
        display_tab.append(&colorspace_box);
        notebook.append_page(
            &display_tab,
            Some(&Label::new(Some("Display"))),
        );
    }

    fn create_performance_tab(&self) -> GtkBox {
        let vbox = GtkBox::new(Orientation::Vertical, 10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);
        vbox.set_margin_top(10);
        vbox.set_margin_bottom(10);

        // GPU Acceleration
        let gpu_box = GtkBox::new(Orientation::Horizontal, 10);
        let gpu_label = Label::new(Some("Use GPU Acceleration"));
        let gpu_switch = Switch::new();
        gpu_switch.set_active(self.settings.borrow().performance.use_gpu);
        gpu_box.append(&gpu_label);
        gpu_box.append(&gpu_switch);
        vbox.append(&gpu_box);

        // Memory Usage
        let memory_box = GtkBox::new(Orientation::Horizontal, 10);
        let memory_label = Label::new(Some("Memory Usage (%)"));
        let memory_scale = Scale::with_range(Orientation::Horizontal, 1.0, 32.0, 1.0);
        memory_scale.set_value(self.settings.borrow().performance.memory_usage_percent as f64);
        memory_box.append(&memory_label);
        memory_box.append(&memory_scale);
        vbox.append(&memory_box);

        // Undo Levels
        let undo_box = GtkBox::new(Orientation::Horizontal, 10);
        let undo_label = Label::new(Some("Undo Levels"));
        let undo_scale = Scale::with_range(Orientation::Horizontal, 1.0, 100.0, 1.0);
        undo_scale.set_value(self.settings.borrow().performance.undo_levels as f64);
        undo_box.append(&undo_label);
        undo_box.append(&undo_scale);
        vbox.append(&undo_box);

        vbox
    }

    fn create_save_tab(&self) -> GtkBox {
        let vbox = GtkBox::new(Orientation::Vertical, 10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);
        vbox.set_margin_top(10);
        vbox.set_margin_bottom(10);

        // Default Format
        let format_box = GtkBox::new(Orientation::Horizontal, 10);
        let format_label = Label::new(Some("Default Format"));
        let format_combo = ComboBoxText::new();
        format_combo.append(Some("jpg"), "JPEG");
        format_combo.append(Some("png"), "PNG");
        format_combo.append(Some("webp"), "WebP");
        format_combo.set_active_id(Some(&self.settings.borrow().save.default_format));
        format_box.append(&format_label);
        format_box.append(&format_combo);
        vbox.append(&format_box);

        // Include Metadata
        let metadata_box = GtkBox::new(Orientation::Horizontal, 10);
        let metadata_label = Label::new(Some("Include Metadata"));
        let metadata_switch = Switch::new();
        metadata_switch.set_active(self.settings.borrow().save.include_metadata);
        metadata_box.append(&metadata_label);
        metadata_box.append(&metadata_switch);
        vbox.append(&metadata_box);

        vbox
    }

    fn create_display_tab(&self) -> GtkBox {
        let vbox = GtkBox::new(Orientation::Vertical, 10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);
        vbox.set_margin_top(10);
        vbox.set_margin_bottom(10);

        // Use Checkerboard
        let checkerboard_box = GtkBox::new(Orientation::Horizontal, 10);
        let checkerboard_label = Label::new(Some("Show Transparency Checkerboard"));
        let checkerboard_switch = Switch::new();
        checkerboard_switch.set_active(self.settings.borrow().display.use_checkerboard);
        checkerboard_box.append(&checkerboard_label);
        checkerboard_box.append(&checkerboard_switch);
        vbox.append(&checkerboard_box);

        // Color Space
        let colorspace_box = GtkBox::new(Orientation::Horizontal, 10);
        let colorspace_label = Label::new(Some("Color Space"));
        let colorspace_combo = ComboBoxText::new();
        colorspace_combo.append(Some("srgb"), "sRGB");
        colorspace_combo.append(Some("adobe"), "Adobe RGB");
        colorspace_combo.append(Some("prophoto"), "ProPhoto RGB");
        colorspace_combo.set_active_id(Some(&self.settings.borrow().display.color_space));
        colorspace_box.append(&colorspace_label);
        colorspace_box.append(&colorspace_combo);
        vbox.append(&colorspace_box);

        vbox
    }
} 