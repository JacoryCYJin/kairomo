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
    BOOL dark = [[self.effectiveAppearance bestMatchFromAppearancesWithNames:
        @[NSAppearanceNameAqua, NSAppearanceNameDarkAqua]] isEqualToString:NSAppearanceNameDarkAqua];
    NSColor *color = NSColor.labelColor;
    NSColor *background = nil;
    if (self.style == 1) {
        background = [NSColor colorWithSRGBRed:67.0/255 green:41.0/255 blue:28.0/255 alpha:1];
        color = [NSColor colorWithSRGBRed:1 green:229.0/255 blue:201.0/255 alpha:1];
    } else if (self.style == 2) {
        background = dark
            ? [NSColor colorWithSRGBRed:69.0/255 green:55.0/255 blue:43.0/255 alpha:1]
            : [NSColor colorWithSRGBRed:243.0/255 green:231.0/255 blue:217.0/255 alpha:1];
        color = dark
            ? [NSColor colorWithSRGBRed:235.0/255 green:195.0/255 blue:152.0/255 alpha:1]
            : [NSColor colorWithSRGBRed:118.0/255 green:81.0/255 blue:53.0/255 alpha:1];
    }
    CGFloat centerY = NSMidY(self.bounds);
    if (background) {
        // Fill only. No stroke, shadow, or state-dependent layout inset.
        [background setFill];
        [[NSBezierPath bezierPathWithRoundedRect:
            NSMakeRect(0, centerY - 10, self.bounds.size.width, 20)
            xRadius:5 yRadius:5] fill];
    }
    NSFont *font = [NSFont monospacedDigitSystemFontOfSize:13 weight:NSFontWeightRegular];
    NSDictionary *attributes = @{NSFontAttributeName:font, NSForegroundColorAttributeName:color};
    NSAttributedString *reference = [[NSAttributedString alloc] initWithString:@"00:00" attributes:attributes];
    CTLineRef referenceLine = CTLineCreateWithAttributedString((__bridge CFAttributedStringRef)reference);
    CGRect digitBounds = CTLineGetImageBounds(referenceLine, NULL);
    CFRelease(referenceLine);
    // These anchors depend only on the format, never the current glyphs or state.
    CGFloat timeX = kairomoTrayPadding + kairomoTrayIconWidth + kairomoTrayGap;
    CGFloat iconRight = kairomoTrayPadding + kairomoTrayIconWidth;
    if (self.style == 2) {
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
        CGContextSetTextPosition(context, iconRight - CGRectGetMaxX(bounds), centerY - CGRectGetMidY(bounds));
        CTLineDraw(plusLine, context);
        CGContextRestoreGState(context);
        CFRelease(plusLine);
    } else {
        NSImage *icon = self.style == 1 ? self.restIcon : self.focusIcon;
        if (self.style == 0) {
            NSImage *source = icon;
            icon = [NSImage imageWithSize:source.size flipped:NO drawingHandler:^BOOL(NSRect rect) {
                [source drawInRect:rect];
                [color setFill];
                NSRectFillUsingOperation(rect, NSCompositingOperationSourceAtop);
                return YES;
            }];
        }
        // Original marks keep their scale. Their visible right edge shares a
        // fixed anchor so a smaller plus cannot create a larger internal gap.
        [icon drawInRect:NSMakeRect(iconRight - icon.size.width,
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
