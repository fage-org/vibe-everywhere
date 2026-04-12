import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import type { Notification } from "./components/routes";
import type { AppShellState } from "./useAppShellState";

function feedTitle(
  item: AppShellState["feedItems"][number],
  shell: AppShellState,
  t: (key: string, options?: Record<string, unknown>) => string,
): string {
  const body = item.body;
  switch (body.kind) {
    case "text":
      return body.text;
    case "friend_request":
    case "friend_accepted": {
      const relatedUser = shell.friends.find((friend) => friend.id === body.uid);
      const userLabel = relatedUser?.firstName || relatedUser?.username || body.uid;
      return body.kind === "friend_request"
        ? t("routes:inbox.feed.friendRequest.title", { userLabel })
        : t("routes:inbox.feed.friendAccepted.title", { userLabel });
    }
    default:
      return t("routes:inbox.feed.accountActivity");
  }
}

function feedMessage(
  item: AppShellState["feedItems"][number],
  shell: AppShellState,
  t: (key: string, options?: Record<string, unknown>) => string,
): string {
  const body = item.body;
  switch (body.kind) {
    case "text":
      return new Date(item.createdAt).toLocaleString();
    case "friend_request":
    case "friend_accepted": {
      const relatedUser = shell.friends.find((friend) => friend.id === body.uid);
      const userLabel = relatedUser?.username ? `@${relatedUser.username}` : body.uid;
      return body.kind === "friend_request"
        ? t("routes:inbox.feed.friendRequest.message", { userLabel })
        : t("routes:inbox.feed.friendAccepted.message", { userLabel });
    }
    default:
      return new Date(item.createdAt).toLocaleString();
  }
}

function feedActionPath(item: AppShellState["feedItems"][number]): string | undefined {
  const body = item.body;
  switch (body.kind) {
    case "friend_request":
    case "friend_accepted":
      return `/(app)/user/${body.uid}`;
    case "text":
    default:
      return undefined;
  }
}

export function useInboxFeedAdapter(
  shell: AppShellState,
  navigate: (path: string) => void,
): Notification[] {
  const { t } = useTranslation("routes");

  return useMemo(() => {
    return shell.feedItems.map((item) => {
      const actionPath = feedActionPath(item);
      return {
        id: item.id,
        title: feedTitle(item, shell, t),
        message: feedMessage(item, shell, t),
        type: item.body.kind === "text" ? "info" : "system",
        createdAt: new Date(item.createdAt),
        action: actionPath
          ? {
              label: t("inbox.feed.actionLabel"),
              onClick: () => navigate(actionPath),
            }
          : undefined,
      } satisfies Notification;
    });
  }, [navigate, shell, t]);
}
