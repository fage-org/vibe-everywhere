import { Body } from "../../components/ui/Typography";
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Input, TextArea } from "../../components/ui";
import { useTranslation } from "react-i18next";

type NewSessionRouteProps = {
  workspace: string;
  model: string;
  title: string;
  prompt: string;
  validationErrors?: {
    workspace?: string;
    prompt?: string;
  };
  formError?: string | null;
  isCreatingSession: boolean;
  onWorkspaceChange: (value: string) => void;
  onModelChange: (value: string) => void;
  onTitleChange: (value: string) => void;
  onPromptChange: (value: string) => void;
  onCreateSession: () => Promise<void> | void;
  onBack: () => void;
  titleText: string;
};

export function NewSessionRoute({
  workspace,
  model,
  title,
  prompt,
  validationErrors,
  formError,
  isCreatingSession,
  onWorkspaceChange,
  onModelChange,
  onTitleChange,
  onPromptChange,
  onCreateSession,
  onBack,
  titleText,
}: NewSessionRouteProps) {
  const { t } = useTranslation(["routes", "common"]);

  return (
    <Card variant="default" style={{ maxWidth: "820px", margin: "0 auto" }}>
      <CardHeader>
        <CardTitle>{titleText}</CardTitle>
        <CardDescription>{t("routes:newSession.description")}</CardDescription>
      </CardHeader>
      <CardContent style={{ display: "flex", flexDirection: "column", gap: 16 }}>
        <Input
          label={t("routes:newSession.fields.workspace.label")}
          value={workspace}
          onChange={(event) => onWorkspaceChange(event.target.value)}
          placeholder={t("routes:newSession.fields.workspace.placeholder")}
          error={validationErrors?.workspace}
          fullWidth
        />
        <Input
          label={t("routes:newSession.fields.model.label")}
          value={model}
          onChange={(event) => onModelChange(event.target.value)}
          placeholder={t("routes:newSession.fields.model.placeholder")}
          fullWidth
        />
        <Input
          label={t("routes:newSession.fields.title.label")}
          value={title}
          onChange={(event) => onTitleChange(event.target.value)}
          placeholder={t("routes:newSession.fields.title.placeholder")}
          fullWidth
        />
        <TextArea
          label={t("routes:newSession.fields.prompt.label")}
          value={prompt}
          onChange={(event) => onPromptChange(event.target.value)}
          minRows={6}
          placeholder={t("routes:newSession.fields.prompt.placeholder")}
          fullWidth
          error={validationErrors?.prompt}
        />
        {formError ? <Body color="secondary">{formError}</Body> : null}
        <div style={{ display: "flex", gap: 12 }}>
          <Button variant="primary" onClick={onCreateSession} loading={isCreatingSession}>
            {titleText}
          </Button>
          <Button variant="ghost" onClick={onBack}>
            {t("common:back")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
