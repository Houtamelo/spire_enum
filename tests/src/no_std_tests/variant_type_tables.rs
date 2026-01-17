use super::*;

#[delegated_enum(extract_variants(derive(Debug, Clone, Default, PartialEq)))]
#[variant_type_table(
    mod_name = custom_table,
    ty_name = SettingsTable,
    attrs(derive(Debug, Default)),
    derive(Clone),
)]
pub enum SettingsEnum {
    MaxFps(i32),
    DialogueTextSpeed(i32),
    Vsync(bool),
    MainVolume(i32),
    MusicVolume(i32),
    SfxVolume(i32),
    VoiceVolume(i32),
}

#[test]
fn test() {
    let mut table = SettingsTable::default();
    *table.get_mut::<MaxFps>() = MaxFps(60);
    *table.get_mut::<MainVolume>() = MainVolume(10);
    *table.get_mut::<Vsync>() = Vsync(false);

    assert_eq!(*table.get::<MaxFps>(), MaxFps(60));
    assert_eq!(*table.get::<MainVolume>(), MainVolume(10));
    assert_eq!(*table.get::<Vsync>(), Vsync(false));

    // Ensure clone works
    let table_clone = table.clone();
    assert_eq!(*table_clone.get::<MaxFps>(), MaxFps(60));
    assert_eq!(*table_clone.get::<MainVolume>(), MainVolume(10));
    assert_eq!(*table_clone.get::<Vsync>(), Vsync(false));

    // Ensure debug is implemented.
    let _ = <SettingsTable as Debug>::fmt;

    table.set(Vsync(true));
    assert_eq!(*table.get::<Vsync>(), Vsync(true));

    table.set_enum(SettingsEnum::MaxFps(MaxFps(25)));
    assert_eq!(*table.get::<MaxFps>(), MaxFps(25));
}
