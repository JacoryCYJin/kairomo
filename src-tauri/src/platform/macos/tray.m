#import <AppKit/AppKit.h>
#import <CoreText/CoreText.h>
#import <objc/runtime.h>

static const CGFloat kairomoTrayPadding = 2;
static const CGFloat kairomoTrayIconWidth = 16;
static const CGFloat kairomoTrayGap = 2;

// Remove transparent padding without changing the mark's original 18pt scale.
// The resulting image's bounds describe the visible shape, not its PNG canvas.
static NSImage *kairomo_trim_tray_icon(NSData *data) {
    NSBitmapImageRep *bitmap = [NSBitmapImageRep imageRepWithData:data];
    NSInteger left = bitmap.pixelsWide, top = bitmap.pixelsHigh, right = -1, bottom = -1;
    for (NSInteger y = 0; y < bitmap.pixelsHigh; y++) {
        for (NSInteger x = 0; x < bitmap.pixelsWide; x++) {
            if ([bitmap colorAtX:x y:y].alphaComponent > 0) {
                left = MIN(left, x); right = MAX(right, x);
                top = MIN(top, y); bottom = MAX(bottom, y);
            }
        }
    }
    if (right < left || bottom < top) return nil;
    CGRect bounds = CGRectMake(left, top, right - left + 1, bottom - top + 1);
    CGImageRef cropped = CGImageCreateWithImageInRect(bitmap.CGImage, bounds);
    NSImage *image = [[NSImage alloc] initWithCGImage:cropped
        size:NSMakeSize(bounds.size.width * 18 / bitmap.pixelsWide,
                        bounds.size.height * 18 / bitmap.pixelsHigh)];
    CGImageRelease(cropped);
    return image;
}

static CGFloat kairomo_tray_width(BOOL hours) {
    NSFont *font = [NSFont monospacedDigitSystemFontOfSize:13 weight:NSFontWeightRegular];
    NSString *sample = hours ? @"00:00:00" : @"00:00";
    // Icon, gap, and margins stay identical; only the time format changes width.
    return 2 * kairomoTrayPadding + kairomoTrayIconWidth + kairomoTrayGap
        + ceil([sample sizeWithAttributes:@{NSFontAttributeName:font}].width);
}

// Decorative content only: the original status button and tray target continue
// to own mouse events, highlighting, menus, and the popup anchor.
@interface KairomoTrayContent : NSView
@property(nonatomic, copy) NSString *time;
@property(nonatomic) NSInteger style;
@property(nonatomic, strong) NSImage *focusIcon;
@property(nonatomic, strong) NSImage *restIcon;
@end

@implementation KairomoTrayContent
- (NSView *)hitTest:(NSPoint)point { return nil; }
- (BOOL)isAccessibilityElement { return NO; }
- (void)viewDidChangeEffectiveAppearance {
    [super viewDidChangeEffectiveAppearance];
    self.needsDisplay = YES;
}
- (void)drawRect:(NSRect)dirtyRect {
    NSColor *color = NSColor.labelColor;
    NSStatusBarButton *button = (NSStatusBarButton *)self.superview;
    if (button.highlighted) color = NSColor.selectedMenuItemTextColor;
    CGFloat centerY = NSMidY(self.bounds);
    CGFloat centerX = kairomoTrayPadding + kairomoTrayIconWidth / 2;
    if (self.style == 2) {
        NSBezierPath *plus = [NSBezierPath bezierPath];
        plus.lineWidth = 1.5;
        plus.lineCapStyle = NSLineCapStyleRound;
        [plus moveToPoint:NSMakePoint(centerX - 5, centerY)];
        [plus lineToPoint:NSMakePoint(centerX + 5, centerY)];
        [plus moveToPoint:NSMakePoint(centerX, centerY - 5)];
        [plus lineToPoint:NSMakePoint(centerX, centerY + 5)];
        [color setStroke];
        [plus stroke];
    } else {
        NSImage *icon = self.style == 1 ? self.restIcon : self.focusIcon;
        // Tint the focus mark inside its own image so compositing cannot affect
        // the transparent menu-bar background. Keep the rest icon's amber.
        if (self.style == 0) {
            NSImage *source = icon;
            icon = [NSImage imageWithSize:source.size flipped:NO
                drawingHandler:^BOOL(NSRect rect) {
                    [source drawInRect:rect];
                    [color setFill];
                    NSRectFillUsingOperation(rect, NSCompositingOperationSourceAtop);
                    return YES;
                }];
        }
        [icon drawInRect:NSMakeRect(centerX - icon.size.width / 2,
                                   centerY - icon.size.height / 2,
                                   icon.size.width, icon.size.height)];
    }
    NSFont *font = [NSFont monospacedDigitSystemFontOfSize:13 weight:NSFontWeightRegular];
    NSDictionary *attributes = @{NSFontAttributeName:font,
        NSForegroundColorAttributeName:color};
    NSAttributedString *reference = [[NSAttributedString alloc] initWithString:@"00:00" attributes:attributes];
    CTLineRef referenceLine = CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)reference);
    CGRect digitBounds = CTLineGetImageBounds(referenceLine, NULL);
    CFRelease(referenceLine);
    NSAttributedString *text = [[NSAttributedString alloc] initWithString:self.time attributes:attributes];
    CTLineRef line = CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)text);
    CGContextRef context = NSGraphicsContext.currentContext.CGContext;
    CGContextSaveGState(context);
    CGContextSetTextMatrix(context, CGAffineTransformIdentity);
    // Anchor to a stable reference digit outline, excluding invisible line
    // leading and descenders. Changing seconds cannot change the baseline.
    CGContextSetTextPosition(context,
        self.bounds.size.width - kairomoTrayPadding - CTLineGetTypographicBounds(line, NULL, NULL, NULL),
        centerY - CGRectGetMidY(digitBounds));
    CTLineDraw(line, context);
    CGContextRestoreGState(context);
    CFRelease(line);
}
@end

void kairomo_update_tray(void *pointer, const char *label, int style,
                         const unsigned char *focus, size_t focusLength,
                         const unsigned char *rest, size_t restLength) {
    NSStatusItem *item = (__bridge NSStatusItem *)pointer;
    NSStatusBarButton *button = item.button;
    if (!button) return;
    static char contentKey;
    NSString *time = [NSString stringWithUTF8String:label];
    CGFloat width = kairomo_tray_width(time.length == 8);
    KairomoTrayContent *content = objc_getAssociatedObject(item, &contentKey);
    if (!content) {
        content = [[KairomoTrayContent alloc] initWithFrame:NSMakeRect(0, 0, width, button.bounds.size.height)];
        content.focusIcon = kairomo_trim_tray_icon([NSData dataWithBytes:focus length:focusLength]);
        content.restIcon = kairomo_trim_tray_icon([NSData dataWithBytes:rest length:restLength]);
        content.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
        button.title = @"";
        button.image = nil;
        [button addSubview:content positioned:NSWindowBelow relativeTo:nil];
        objc_setAssociatedObject(item, &contentKey, content, OBJC_ASSOCIATION_RETAIN_NONATOMIC);
    }
    if (item.length != width) {
        item.length = width;
        // Keep the original tray-icon event target and decorative content sized
        // together in both directions across the one-hour format boundary.
        for (NSView *target in button.subviews) {
            target.frame = NSMakeRect(0, 0, width, button.bounds.size.height);
            target.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
        }
    }
    if (![content.time isEqualToString:time] || content.style != style) {
        content.time = time;
        content.style = style;
        content.needsDisplay = YES;
        [button setAccessibilityLabel:[NSString stringWithFormat:@"%@ %@",
            style == 2 ? @"+" : (style == 1 ? @"Rest" : @"Focus"), time]];
    }
}
