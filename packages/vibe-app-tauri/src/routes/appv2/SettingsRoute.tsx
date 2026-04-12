import { SettingsSurface, type SettingSection } from "../../components/routes";
import { useTranslation } from "react-i18next";

type SettingsRouteProps = {
  sections: SettingSection[];
};

export function SettingsRoute({ sections }: SettingsRouteProps) {
  const { t } = useTranslation("routes");

  return (
    <SettingsSurface
      sections={sections}
      description={t("settings.appV2Persistence")}
    />
  );
}
