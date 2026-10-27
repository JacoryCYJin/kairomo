#import <AppKit/AppKit.h>
#import <objc/runtime.h>

// The web toolbar is 48 logical points high. Position the real system buttons
// in that same coordinate space, rather than relying on a titlebar inset.
@interface KairomoTitlebarAlignment : NSObject
@property(nonatomic, weak) NSWindow *window;
@property(nonatomic, strong) NSMutableArray *observers;
@property(nonatomic) BOOL pending;
- (void)align;
- (void)schedule;
@end

@implementation KairomoTitlebarAlignment
- (void)align {
    NSWindow *window = self.window;
    if (!window || (window.styleMask & NSWindowStyleMaskFullScreen)) return;
    NSButton *close = [window standardWindowButton:NSWindowCloseButton];
    NSView *container = close.superview.superview;
    if (!container) return;
    NSRect frame = container.frame;
    frame.size.height = 48.0;
    frame.origin.y = NSHeight(window.frame) - 48.0;
    if (!NSEqualRects(container.frame, frame)) container.frame = frame;

    NSArray *kinds = @[@(NSWindowCloseButton), @(NSWindowMiniaturizeButton), @(NSWindowZoomButton)];
    for (NSUInteger index = 0; index < kinds.count; index++) {
        NSButton *button = [window standardWindowButton:[kinds[index] unsignedIntegerValue]];
        if (!button.superview) continue;
        NSPoint center = NSMakePoint(15.0 + index * 23.0 + NSWidth(button.frame) / 2.0, 24.0);
        center = [button.superview convertPoint:center fromView:container];
        NSPoint origin = NSMakePoint(center.x - NSWidth(button.frame) / 2.0,
                                     center.y - NSHeight(button.frame) / 2.0);
        if (!NSEqualPoints(button.frame.origin, origin)) [button setFrameOrigin:origin];
    }
}
- (void)schedule {
    if (self.pending) return;
    self.pending = YES;
    __weak KairomoTitlebarAlignment *weakSelf = self;
    dispatch_async(dispatch_get_main_queue(), ^{
        KairomoTitlebarAlignment *strongSelf = weakSelf;
        strongSelf.pending = NO;
        [strongSelf align];
    });
}
- (void)dealloc {
    for (id observer in self.observers) [[NSNotificationCenter defaultCenter] removeObserver:observer];
}
@end

void kairomo_align_details_titlebar(void *pointer) {
    NSWindow *window = (__bridge NSWindow *)pointer;
    static char alignmentKey;
    KairomoTitlebarAlignment *alignment = objc_getAssociatedObject(window, &alignmentKey);
    if (!alignment) {
        alignment = [KairomoTitlebarAlignment new];
        alignment.window = window;
        alignment.observers = [NSMutableArray new];
        objc_setAssociatedObject(window, &alignmentKey, alignment, OBJC_ASSOCIATION_RETAIN_NONATOMIC);
        __weak KairomoTitlebarAlignment *weakAlignment = alignment;
        NSArray *names = @[NSWindowDidResizeNotification, NSWindowDidBecomeKeyNotification,
                           NSWindowDidDeminiaturizeNotification, NSWindowDidExitFullScreenNotification,
                           NSWindowDidUpdateNotification];
        for (NSNotificationName name in names) {
            id observer = [[NSNotificationCenter defaultCenter] addObserverForName:name object:window
                queue:nil usingBlock:^(NSNotification *notification) {
                    (void)notification;
                    [weakAlignment schedule];
                }];
            [alignment.observers addObject:observer];
        }
    }
    [alignment align];
    [alignment schedule];
}
