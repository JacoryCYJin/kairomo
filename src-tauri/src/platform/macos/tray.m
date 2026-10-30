#import <AppKit/AppKit.h>
#import <CoreText/CoreText.h>
#import <objc/runtime.h>

static const CGFloat kairomoTrayPadding = 5;
static const CGFloat kairomoTrayIconWidth = 15;
static const CGFloat kairomoTrayGap = 3;

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

static NSColor *kairomo_theme_color(NSData *palette, NSUInteger offset) {
    const unsigned char *rgb = (const unsigned char *)palette.bytes + offset;
    return [NSColor colorWithSRGBRed:rgb[0]/255.0 green:rgb[1]/255.0 blue:rgb[2]/255.0 alpha:1];
}

static CGFloat kairomo_tray_width(BOOL hours) {
    NSFont *font = [NSFont monospacedDigitSystemFontOfSize:13 weight:NSFontWeightRegular];
    NSString *sample = hours ? @"00:00:00" : @"00:00";
    NSAttributedString *text = [[NSAttributedString alloc] initWithString:sample
        attributes:@{NSFontAttributeName:font}];
    CTLineRef line = CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)text);
    CGFloat origin = kairomoTrayPadding + kairomoTrayIconWidth + kairomoTrayGap
        - CGRectGetMinX(CTLineGetImageBounds(line, NULL));
    CFRelease(line);
    CGFloat minimum = CGFLOAT_MAX, maximum = 0;
    for (int digit = 0; digit < 10; digit++) {
        NSString *label = [[sample substringToIndex:sample.length - 1]
            stringByAppendingFormat:@"%d", digit];
        NSAttributedString *text = [[NSAttributedString alloc] initWithString:label
            attributes:@{NSFontAttributeName:font}];
        CTLineRef variant = CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)text);
        CGFloat right = CGRectGetMaxX(CTLineGetImageBounds(variant, NULL));
        CFRelease(variant);
        minimum = MIN(minimum, right);
        maximum = MAX(maximum, right);
    }
    // One stable width per format. Center the natural trailing digit bearings
    // around the 5pt visible inset instead of retaining the font's blank advance.
    return origin + (minimum + maximum) / 2 + kairomoTrayPadding;
}

// Decorative content only: the original status button and tray target continue
// to own mouse events, highlighting, menus, and the popup anchor.
@interface KairomoTrayContent : NSView
@property(nonatomic, copy) NSString *time;
@property(nonatomic) NSInteger style;
@property(nonatomic) NSInteger restAppearance;
@property(nonatomic, strong) NSImage *focusIcon;
@property(nonatomic, strong) NSImage *restIcon;
@property(nonatomic, strong) NSImage *overtimeIcon;
@property(nonatomic, strong) NSData *focusIconData;
@property(nonatomic, strong) NSData *overtimeIconData;
@property(nonatomic) BOOL focusBackground;
@property(nonatomic, strong) NSData *restIconData;
@property(nonatomic, strong) NSData *palette;
@end

@implementation KairomoTrayContent
- (NSView *)hitTest:(NSPoint)point { return nil; }
- (BOOL)isAccessibilityElement { return NO; }
- (void)viewDidChangeEffectiveAppearance {
    [super viewDidChangeEffectiveAppearance];
    self.needsDisplay = YES;
}
- (void)drawRect:(NSRect)dirtyRect {
    // Explicit appearance follows app settings. System mode follows the app,
    // even when the desktop wallpaper makes the menu bar look different.
    BOOL appDark = [[NSApp.effectiveAppearance bestMatchFromAppearancesWithNames:
        @[NSAppearanceNameAqua, NSAppearanceNameDarkAqua]] isEqualToString:NSAppearanceNameDarkAqua];
    BOOL dark = self.restAppearance == 2 || (self.restAppearance == 0 && appDark);
    NSColor *color = NSColor.labelColor;
    NSColor *background = nil;
    if (self.style != 0 || self.focusBackground) {
        NSUInteger offset = (dark ? 18 : 0) + self.style * 6;
        background = kairomo_theme_color(self.palette, offset);
        color = kairomo_theme_color(self.palette, offset + 3);
    }
    CGFloat centerY = NSMidY(self.bounds);
    if (background) {
        // Fill only. No stroke, shadow, or state-dependent layout inset.
        [background setFill];
        [[NSBezierPath bezierPathWithRoundedRect:
            NSMakeRect(0, centerY - 10, self.bounds.size.width, 20)
            xRadius:5 yRadius:5] fill];
    } else {
        [color setStroke];
        NSBezierPath *outline = [NSBezierPath bezierPathWithRoundedRect:
            NSMakeRect(0.5, centerY - 9.5, self.bounds.size.width - 1, 19)
            xRadius:4.5 yRadius:4.5];
        outline.lineWidth = 1;
        [outline stroke];
    }
    NSFont *font = [NSFont monospacedDigitSystemFontOfSize:13 weight:NSFontWeightRegular];
    NSDictionary *attributes = @{NSFontAttributeName:font, NSForegroundColorAttributeName:color};
    NSAttributedString *reference = [[NSAttributedString alloc] initWithString:@"00:00" attributes:attributes];
    CTLineRef referenceLine = CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)reference);
    CGRect digitBounds = CTLineGetImageBounds(referenceLine, NULL);
    CFRelease(referenceLine);
    // These anchors depend only on the format, never the current glyphs or state.
    CGFloat timeX = kairomoTrayPadding + kairomoTrayIconWidth + kairomoTrayGap;
    if (self.style == 2 && !self.overtimeIcon) {
        NSFont *sampleFont = [NSFont systemFontOfSize:25 weight:NSFontWeightRegular];
        NSAttributedString *sample = [[NSAttributedString alloc] initWithString:@"+"
            attributes:@{NSFontAttributeName:sampleFont}];
        CTLineRef sampleLine = CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)sample);
        CGRect sampleBounds = CTLineGetImageBounds(sampleLine, NULL);
        CFRelease(sampleLine);
        NSFont *plusFont = [NSFont systemFontOfSize:25 * 9 / sampleBounds.size.height weight:NSFontWeightRegular];
        NSAttributedString *plus = [[NSAttributedString alloc] initWithString:@"+"
            attributes:@{NSFontAttributeName:plusFont, NSForegroundColorAttributeName:color}];
        CTLineRef plusLine = CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)plus);
        CGRect bounds = CTLineGetImageBounds(plusLine, NULL);
        CGContextRef context = NSGraphicsContext.currentContext.CGContext;
        CGContextSaveGState(context);
        CGContextSetTextMatrix(context, CGAffineTransformIdentity);
        CGContextSetTextPosition(context, kairomoTrayPadding - CGRectGetMinX(bounds), centerY - CGRectGetMidY(bounds));
        CTLineDraw(plusLine, context);
        CGContextRestoreGState(context);
        CFRelease(plusLine);
    } else {
        NSImage *source = self.style == 1 ? self.restIcon :
            (self.style == 2 ? self.overtimeIcon : self.focusIcon);
        NSImage *icon = [NSImage imageWithSize:source.size flipped:NO drawingHandler:^BOOL(NSRect rect) {
            [source drawInRect:rect];
            [color setFill];
            NSRectFillUsingOperation(rect, NSCompositingOperationSourceAtop);
            return YES;
        }];
        // Cropped visible ink starts at the same outer inset as the digits end.
        // Preserve the icon's scale and the fixed 15pt slot / digit anchor.
        [icon drawInRect:NSMakeRect(kairomoTrayPadding,
            centerY - icon.size.height / 2, icon.size.width, icon.size.height)];
    }
    NSAttributedString *text = [[NSAttributedString alloc] initWithString:self.time attributes:attributes];
    CTLineRef line = CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)text);
    CGContextRef context = NSGraphicsContext.currentContext.CGContext;
    CGContextSaveGState(context);
    CGContextSetTextMatrix(context, CGAffineTransformIdentity);
    CGContextSetTextPosition(context, timeX - CGRectGetMinX(digitBounds), centerY - CGRectGetMidY(digitBounds));
    CTLineDraw(line, context);
    CGContextRestoreGState(context);
    CFRelease(line);
}
@end

void kairomo_update_tray(void *pointer, const char *label, int style, int restAppearance,
                         const unsigned char *focus, size_t focusLength,
                         const unsigned char *rest, size_t restLength,
                         const unsigned char *overtime, size_t overtimeLength,
                         const unsigned char *palette, bool focusBackground) {
    NSStatusItem *item = (__bridge NSStatusItem *)pointer;
    NSStatusBarButton *button = item.button;
    if (!button) return;
    static char contentKey;
    NSString *time = [NSString stringWithUTF8String:label];
    CGFloat width = kairomo_tray_width(time.length == 8);
    KairomoTrayContent *content = objc_getAssociatedObject(item, &contentKey);
    if (!content) {
        content = [[KairomoTrayContent alloc] initWithFrame:NSMakeRect(0, 0, width, button.bounds.size.height)];
        content.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
        button.title = @"";
        button.image = nil;
        [button addSubview:content positioned:NSWindowBelow relativeTo:nil];
        objc_setAssociatedObject(item, &contentKey, content, OBJC_ASSOCIATION_RETAIN_NONATOMIC);
    }
    NSData *nextPalette = [NSData dataWithBytes:palette length:36];
    NSData *nextFocus = [NSData dataWithBytes:focus length:focusLength];
    NSData *nextOvertime = [NSData dataWithBytes:overtime length:overtimeLength];
    if (![content.focusIconData isEqualToData:nextFocus]) {
        content.focusIconData = nextFocus;
        content.focusIcon = kairomo_trim_tray_icon(nextFocus);
        content.needsDisplay = YES;
    }
    if (![content.overtimeIconData isEqualToData:nextOvertime]) {
        content.overtimeIconData = nextOvertime;
        content.overtimeIcon = overtimeLength ? kairomo_trim_tray_icon(nextOvertime) : nil;
        content.needsDisplay = YES;
    }
    if (content.focusBackground != focusBackground) {
        content.focusBackground = focusBackground;
        content.needsDisplay = YES;
    }
    NSData *nextIcon = [NSData dataWithBytes:rest length:restLength];
    if (![content.restIconData isEqualToData:nextIcon]) {
        content.restIconData = nextIcon;
        content.restIcon = kairomo_trim_tray_icon(nextIcon);
        content.needsDisplay = YES;
    }
    if (![content.palette isEqualToData:nextPalette]) {
        content.palette = nextPalette;
        content.needsDisplay = YES;
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
    if (![content.time isEqualToString:time] || content.style != style
        || content.restAppearance != restAppearance) {
        content.time = time;
        content.style = style;
        content.restAppearance = restAppearance;
        content.needsDisplay = YES;
        [button setAccessibilityLabel:[NSString stringWithFormat:@"%@ %@",
            style == 2 ? @"+" : (style == 1 ? @"Rest" : @"Focus"), time]];
    }
}
