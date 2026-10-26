#import <AppKit/AppKit.h>

// Apply to the NSWindow as well as NSApp so existing title bars update immediately.
void kairomo_set_window_appearance(void *pointer, int mode) {
    NSWindow *window = (__bridge NSWindow *)pointer;
    NSAppearance *appearance = nil;
    if (mode == 1) appearance = [NSAppearance appearanceNamed:NSAppearanceNameAqua];
    if (mode == 2) appearance = [NSAppearance appearanceNamed:NSAppearanceNameDarkAqua];
    window.appearance = appearance;
    window.contentView.superview.needsDisplay = YES;
    [window displayIfNeeded];
}
