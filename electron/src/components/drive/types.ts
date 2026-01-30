// Types for Drive/Servo Control

export interface DriveState {
  name: string;
  position: number;
  setpointPosition: number;
  velocity: number;
  errorCode: number;
  lagError: {
    current: number;
    min: number;
    max: number;
  };
  actualVelocity: number;
  setpointVelocity: number;
  override: number; // 0-10000 (100.00 = 100%)
  outputPercent: number;
  controllerOutputPercent: number;
  
  // Status flags (Logical)
  ready: boolean;
  calibrated: boolean;
  hasJob: boolean;
  notMoving: boolean;
  movingForward: boolean;
  movingBackward: boolean;
  
  // Status flags (Physical)
  coupledMode: boolean;
  inTargetPos: boolean;
  inPosRange: boolean;
  
  // Enabling
  controllerEnabled: boolean;
  feedFwEnabled: boolean;
  feedBwEnabled: boolean;
}

export interface DriveControlParams {
  targetPosition: number;
  targetVelocity: number;
  acceleration: number;
  deceleration: number;
  jerk: number;
  enableAcceleration: boolean;
  enableDeceleration: boolean;
  enableJerk: boolean;
}

export type MovementMode = 
  | 'Absolute'
  | 'Relative'
  | 'EndlessPos'
  | 'EndlessNeg'
  | 'Modulo'
  | 'ModuloShortestWay'
  | 'ModuloPosDirection'
  | 'ModuloNegDirection'
  | 'JogPos'
  | 'JogNeg'
  | 'PlusOne'
  | 'PlusTenth'
  | 'PlusHundredth'
  | 'PlusThousandth'
  | 'MinusOne'
  | 'MinusTenth'
  | 'MinusHundredth'
  | 'MinusThousandth'
  | 'ReversingSequence'
  | 'StartStopSequence'
  | 'VeloStepSequence'
  | 'SinusSequenceBode'
  | 'SinusOscillation';

export type RawDriveOutputMode = 'Percent' | 'Velocity';
export type SetPositionMode = 'Absolute' | 'Relative';
export type SetTargetMode = 'Absolute' | 'Relative' | 'EndlessPos' | 'EndlessNeg' | 'Modulo';

export interface MovementModeOption {
  value: MovementMode;
  label: string;
}

export const MOVEMENT_MODES: MovementModeOption[] = [
  { value: 'Absolute', label: 'Absolute' },
  { value: 'Relative', label: 'Relative' },
  { value: 'EndlessPos', label: 'Endless +' },
  { value: 'EndlessNeg', label: 'Endless -' },
  { value: 'Modulo', label: 'Modulo' },
  { value: 'ModuloShortestWay', label: 'Modulo shortest way' },
  { value: 'ModuloPosDirection', label: 'Modulo plus direct.' },
  { value: 'ModuloNegDirection', label: 'Modulo minus direct.' },
  { value: 'JogPos', label: 'Jog +' },
  { value: 'JogNeg', label: 'Jog -' },
  { value: 'PlusOne', label: '+ 1' },
  { value: 'PlusTenth', label: '+ 0.1' },
  { value: 'PlusHundredth', label: '+ 0.01' },
  { value: 'PlusThousandth', label: '+ 0.001' },
  { value: 'MinusOne', label: '- 1' },
  { value: 'MinusTenth', label: '- 0.1' },
  { value: 'MinusHundredth', label: '- 0.01' },
  { value: 'MinusThousandth', label: '- 0.001' },
  { value: 'ReversingSequence', label: 'Reversing sequence' },
  { value: 'StartStopSequence', label: 'Start/Stop sequence' },
  { value: 'VeloStepSequence', label: 'Velo step sequence' },
  { value: 'SinusSequenceBode', label: 'Sinus sequence (Bode)' },
  { value: 'SinusOscillation', label: 'Sinus oscillation' },
];
