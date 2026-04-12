import { InboxSurface, type Notification } from "../../components/routes";

type InboxRouteProps = {
  notifications: Notification[];
  unreadCount?: number;
  supportsUnreadFilter?: boolean;
  filter: "all" | "unread";
  onFilterChange: (filter: "all" | "unread") => void;
};

export function InboxRoute({
  notifications,
  unreadCount,
  supportsUnreadFilter,
  filter,
  onFilterChange,
}: InboxRouteProps) {
  return (
    <InboxSurface
      notifications={notifications}
      unreadCount={unreadCount}
      supportsUnreadFilter={supportsUnreadFilter}
      filter={filter}
      onFilterChange={onFilterChange}
    />
  );
}
