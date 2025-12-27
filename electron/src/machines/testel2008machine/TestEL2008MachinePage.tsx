import { Topbar } from "@/components/Topbar";
import { testEL2008MachineSerialRoute } from "@/routes/routes";
import React from "react";

export function TestEL2008MachinePage() {
  const { serial } = testEL2008MachineSerialRoute.useParams();
  return (
    <Topbar
      pathname={`/_sidebar/machines/testel2008machine/${serial}`}
      items={[
        {
          link: "control",
          activeLink: "control",
          title: "Control",
          icon: "lu:CirclePlay",
        },
      ]}
    />
  );
}