/**
 * Example: Using the new Happy-aligned component architecture
 *
 * This file demonstrates how to use the new components
 * to build a session view similar to Happy's UI.
 */

import { useState } from "react";
import { ThemeProvider } from "./components/providers/ThemeProvider";
import { Shell, Sidebar, Header, MobileShell, MobileNavBar } from "./components/layout";
import { Button, Badge } from "./components/ui";
import { SessionList, Timeline, Composer } from "./components/surfaces";
import type { Session, Message, ComposerSuggestion } from "./components/surfaces";

// Example usage of the new component architecture
export function ExampleHappyApp() {
  return (
    <ThemeProvider defaultScheme="dark">
      <DesktopExample />
    </ThemeProvider>
  );
}

// Desktop layout example
function DesktopExample() {
  const [activeTab, setActiveTab] = useState("sessions");
  const [selectedSessionId, setSelectedSessionId] = useState<string>();
  const [composerValue, setComposerValue] = useState("");

  // Example sessions data
  const sessions: Session[] = [
    {
      id: "1",
      title: "Wave 8 Planning",
      subtitle: "Discussing the desktop rewrite",
      lastMessage: "Let's focus on the component architecture first...",
      lastActivityAt: new Date(Date.now() - 1000 * 60 * 5),
      status: "active",
      unread: true,
      unreadCount: 3,
    },
    {
      id: "2",
      title: "API Integration",
      subtitle: "Backend connection setup",
      lastMessage: "The endpoints are ready for testing",
      lastActivityAt: new Date(Date.now() - 1000 * 60 * 60 * 2),
      status: "idle",
    },
  ];

  // Example messages
  const messages: Message[] = [
    {
      id: "1",
      role: "user",
      content: "Let's refactor the UI to match Happy's design",
      timestamp: new Date(Date.now() - 1000 * 60 * 10),
    },
    {
      id: "2",
      role: "assistant",
      content: "I'll help you create a pixel-perfect replication of Happy's UI. Let's start with the design tokens and component architecture.",
      timestamp: new Date(Date.now() - 1000 * 60 * 9),
    },
  ];

  // Composer suggestions
  const suggestions: ComposerSuggestion[] = [
    { id: "1", label: "Continue", insertText: "Continue" },
    { id: "2", label: "Explain", insertText: "Can you explain" },
  ];

  // Navigation items
  const primaryNav = [
    {
      title: "Navigation",
      items: [
        { id: "sessions", label: "Sessions", icon: "💬" },
        { id: "inbox", label: "Inbox", icon: "🔔", badge: 2 },
        { id: "settings", label: "Settings", icon: "⚙️" },
      ],
    },
  ];

  return (
    <Shell
      sidebar={
        <Sidebar
          brand={<div style={{ fontWeight: 700, fontSize: "1.25rem" }}>Vibe</div>}
          primarySections={[
            {
              items: primaryNav[0].items.map((item) => ({
                ...item,
                state: item.id === activeTab ? "active" : "default",
                onClick: () => setActiveTab(item.id),
              })),
            },
          ]}
          secondarySections={[
            {
              items: [
                { id: "help", label: "Help & Support", icon: "❓" },
              ],
            },
          ]}
          connectionStatus={
            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              <span style={{ width: 8, height: 8, borderRadius: "50%", backgroundColor: "var(--color-success)" }} />
              <span style={{ fontSize: "0.8125rem", color: "var(--text-tertiary)" }}>Connected</span>
            </div>
          }
        />
      }
      header={
        <Header
          eyebrow="Session"
          title={selectedSessionId ? "Wave 8 Planning" : "Select a Session"}
          subtitle={selectedSessionId ? "3 unread messages" : undefined}
          actions={
            <>
              <Button variant="ghost" size="sm">Share</Button>
              <Button variant="primary" size="sm">New Session</Button>
            </>
          }
        />
      }
    >
      <div style={{ display: "flex", height: "100%" }}>
        {/* Session List */}
        <div style={{ width: "320px", borderRight: "1px solid var(--border-primary)", overflow: "auto" }}>
          <SessionList
            sessions={sessions}
            selectedId={selectedSessionId}
            onSelect={(session) => setSelectedSessionId(session.id)}
          />
        </div>

        {/* Main Content */}
        <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
          {selectedSessionId ? (
            <>
              <Timeline messages={messages} />
              <Composer
                value={composerValue}
                onChange={setComposerValue}
                onSubmit={() => {
                  console.log("Send:", composerValue);
                  setComposerValue("");
                }}
                suggestions={suggestions}
                onSuggestionSelect={(s) => setComposerValue(s.insertText)}
                footer={
                  <div style={{ display: "flex", gap: "8px" }}>
                    <Badge variant="outline" size="sm">gpt-4</Badge>
                    <Badge variant="ghost" size="sm">Workspace: /root/vibe</Badge>
                  </div>
                }
              />
            </>
          ) : (
            <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: "var(--text-tertiary)" }}>
              Select a session to start chatting
            </div>
          )}
        </div>
      </div>
    </Shell>
  );
}

// Mobile layout example
function MobileExample() {
  const [activeTab, setActiveTab] = useState("sessions");

  return (
    <MobileShell
      header={<MobileNavBar title="Vibe" />}
      tabs={[
        { id: "sessions", label: "Sessions", icon: "💬" },
        { id: "inbox", label: "Inbox", icon: "🔔", badge: 2 },
        { id: "settings", label: "Settings", icon: "⚙️" },
      ]}
      activeTab={activeTab}
      onTabChange={setActiveTab}
    >
      <div style={{ padding: "16px" }}>
        <h2>Mobile Content</h2>
        <p>This is the mobile view using the new component architecture.</p>
      </div>
    </MobileShell>
  );
}
