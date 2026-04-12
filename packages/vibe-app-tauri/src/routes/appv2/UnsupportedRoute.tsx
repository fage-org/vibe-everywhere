import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui";

type UnsupportedRouteProps = {
  title: string;
  description: string;
};

export function UnsupportedRoute({ title, description }: UnsupportedRouteProps) {
  return (
    <Card variant="default" style={{ maxWidth: "760px", margin: "0 auto" }}>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        This route is not yet productized in the AppV2 default shell and remains outside the active
        Wave 10 surface contract.
      </CardContent>
    </Card>
  );
}
