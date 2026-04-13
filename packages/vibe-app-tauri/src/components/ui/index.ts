// UI Primitive Components
// Aligned with Happy's design system

export { Button, type ButtonProps, type ButtonVariant, type ButtonSize } from "./Button";
export {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
  type CardProps,
  type CardVariant,
} from "./Card";
export { Input, type InputProps, type InputSize } from "./Input";
export { TextArea, type TextAreaProps, type TextAreaSize } from "./TextArea";
export { Badge, type BadgeProps, type BadgeVariant, type BadgeSize } from "./Badge";
export {
  LargeTitle,
  Title1,
  Title2,
  Title3,
  Headline,
  Body,
  Callout,
  Subheadline,
  Footnote,
  Caption1,
  Caption2,
  Eyebrow,
  type TypographyProps,
} from "./Typography";

// Multi-line text input with keyboard event handling
export {
  MultiTextInput,
  type MultiTextInputProps,
  type MultiTextInputHandle,
  type KeyPressEvent,
  type SupportedKey,
  type TextInputState,
} from "./MultiTextInput";

// Shimmer loading animations
export {
  ShimmerView,
  ShimmerText,
  ShimmerAvatar,
  ShimmerCard,
  type ShimmerViewProps,
  type ShimmerTextProps,
  type ShimmerAvatarProps,
  type ShimmerCardProps,
} from "./ShimmerView";

// Status indicators
export {
  StatusIndicator,
  StatusBadge,
  getStatusLabel,
  type StatusIndicatorProps,
  type StatusBadgeProps,
  type StatusType,
  type StatusSize,
} from "./StatusIndicator";

// Avatar components
export {
  Avatar,
  AvatarGradient,
  AvatarBrutalist,
  type AvatarProps,
  type AvatarGradientProps,
  type AvatarBrutalistProps,
  type AvatarStyle,
} from "./avatar";
