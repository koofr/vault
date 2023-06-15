#import "UINavigationControllerDelegateProxy.h"

@interface UINavigationControllerDelegateProxy()

@property(nonatomic, strong) id<UINavigationControllerDelegate> delegate;
@property(nonatomic, strong) id<UINavigationControllerDelegate> forward;

@end

@implementation UINavigationControllerDelegateProxy

- (instancetype)initWithDelegate:(id<UINavigationControllerDelegate>)delegate forwardTo:(id<UINavigationControllerDelegate>)forward
{
    self.delegate = delegate;
    self.forward = forward;

    return self;
}

- (BOOL)respondsToSelector:(SEL)aSelector {
    return [self.delegate respondsToSelector:aSelector];
}

- (NSMethodSignature *)methodSignatureForSelector:(SEL)sel {
    return [(id)self.delegate methodSignatureForSelector:sel];
}

- (void)forwardInvocation:(NSInvocation *)invocation {
    if ([self.delegate respondsToSelector:invocation.selector]) {
        [invocation invokeWithTarget:self.delegate];
    }

    if ([self.forward respondsToSelector:invocation.selector]) {
        [invocation invokeWithTarget:self.forward];
    }
}

@end
