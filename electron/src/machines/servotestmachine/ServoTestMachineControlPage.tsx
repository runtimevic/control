import { useParams } from "@tanstack/react-router";
import { ServoTestMachineControl } from "./ServoTestMachineControl";
import { VENDOR_QITECH } from "../properties";

export function ServoTestMachineControlPage() {
  const { serial } = useParams({ strict: false });

  return (
    <ServoTestMachineControl
      machineIdentification={{
        machine_identification: {
          vendor: VENDOR_QITECH,
          machine: 0x0037,
        },
        serial: Number(serial),
      }}
    />
  );
}
