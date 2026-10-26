#import <AppKit/AppKit.h>

typedef struct {
    double x, y, width, height;
    uint32_t display_id;
} KairomoPlacement;
static NSString *const timerOriginName = @"KairomoTimerOrigin";
static void saveOrigin(NSWindow *window) {
    NSPoint origin = window.frame.origin;
    [NSUserDefaults.standardUserDefaults setObject:@{@"x": @(origin.x), @"y": @(origin.y)}
                                           forKey:timerOriginName];
}

KairomoPlacement kairomo_popup_geometry(KairomoPlacement area, double mouse_x, double width, double height) {
    double x = fmax(area.x, fmin(mouse_x - width / 2.0, area.x + area.width - width));
    double y = fmax(area.y, area.y + area.height - 6.0 - height);
    return (KairomoPlacement){x, y, width, height, area.display_id};
}

// A usable strip at the top must remain on a connected display so it can be dragged.
// Partially off-screen placement is retained when the strip is still reachable.
bool kairomo_popup_grip_visible(KairomoPlacement frame, KairomoPlacement area) {
    NSRect grip = NSMakeRect(frame.x, frame.y + frame.height - 32.0, frame.width, 32.0);
    NSRect visible = NSIntersectionRect(grip, NSMakeRect(area.x, area.y, area.width, area.height));
    return visible.size.width >= 48.0 && visible.size.height >= 16.0;
}

static NSScreen *pointerScreen(void) {
    NSPoint pointer = NSEvent.mouseLocation;
    for (NSScreen *screen in NSScreen.screens) {
        if (NSPointInRect(pointer, screen.frame)) return screen;
    }
    return NSScreen.mainScreen;
}
static KairomoPlacement describeFrame(NSRect frame, NSScreen *screen) {
    return (KairomoPlacement){frame.origin.x, frame.origin.y, frame.size.width, frame.size.height,
        [screen.deviceDescription[@"NSScreenNumber"] unsignedIntValue]};
}
static bool positionReachable(NSWindow *window) {
    KairomoPlacement frame = describeFrame(window.frame, window.screen);
    for (NSScreen *screen in NSScreen.screens) {
        if (kairomo_popup_grip_visible(frame, describeFrame(screen.visibleFrame, screen))) return true;
    }
    return false;
}
static bool recoverPosition(NSWindow *window) {
    NSScreen *screen = pointerScreen();
    if (screen == nil) return false;
    NSSize size = window.frame.size;
    KairomoPlacement target = kairomo_popup_geometry(describeFrame(screen.visibleFrame, screen),
        NSEvent.mouseLocation.x, size.width, size.height);
    [window setFrameOrigin:NSMakePoint(target.x, target.y)];
    return true;
}

void kairomo_configure_popup(void *pointer) {
    NSWindow *window = (__bridge NSWindow *)pointer;
    window.styleMask |= NSWindowStyleMaskTitled | NSWindowStyleMaskFullSizeContentView;
    window.titleVisibility = NSWindowTitleHidden;
    window.titlebarAppearsTransparent = YES;
    for (NSNumber *kind in @[@(NSWindowCloseButton), @(NSWindowMiniaturizeButton), @(NSWindowZoomButton)]) {
        [window standardWindowButton:kind.integerValue].hidden = YES;
    }
    window.hidesOnDeactivate = NO;
    window.level = NSFloatingWindowLevel;
    window.collectionBehavior = NSWindowCollectionBehaviorCanJoinAllSpaces |
                                NSWindowCollectionBehaviorFullScreenAuxiliary |
                                NSWindowCollectionBehaviorIgnoresCycle;
    if (@available(macOS 13.0, *)) {
        window.collectionBehavior |= NSWindowCollectionBehaviorCanJoinAllApplications;
    }
    window.hasShadow = YES;
    // AppKit frame autosave may normalize a saved frame onto the window's initial
    // screen. Store the global point origin explicitly instead.
    NSDictionary *saved = [NSUserDefaults.standardUserDefaults objectForKey:timerOriginName];
    BOOL restored = [saved isKindOfClass:NSDictionary.class] &&
        [saved[@"x"] isKindOfClass:NSNumber.class] && [saved[@"y"] isKindOfClass:NSNumber.class] &&
        isfinite([saved[@"x"] doubleValue]) && isfinite([saved[@"y"] doubleValue]);
    if (restored) [window setFrameOrigin:NSMakePoint([saved[@"x"] doubleValue], [saved[@"y"] doubleValue])];
    if (!restored || !positionReachable(window)) recoverPosition(window);
    saveOrigin(window);
    [NSNotificationCenter.defaultCenter addObserverForName:NSWindowDidMoveNotification
        object:window queue:nil usingBlock:^(NSNotification *notification) {
            saveOrigin(notification.object);
        }];
}

bool kairomo_ensure_popup_visible(void *pointer, KairomoPlacement *result) {
    NSWindow *window = (__bridge NSWindow *)pointer;
    if (!positionReachable(window) && !recoverPosition(window)) return false;
    *result = describeFrame(window.frame, window.screen);
    return true;
}
