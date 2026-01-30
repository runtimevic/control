import { Page } from "@/components/Page";
import { Outlet, useParams } from "@tanstack/react-router";

export function ServoTestMachinePage() {
  const { serial } = useParams({ strict: false });

  return (
    <Page>
      <Outlet />
    </Page>
  );
}
