export { WebMessage } from '../bindings/WebMessage';
export { UgenSpec } from '../bindings/UgenSpec';
export { ControlBlock } from '../bindings/ControlBlock';
export { LowpassControlBlock } from '../bindings/LowpassControlBlock';
export { TapType } from '../bindings/TapType';
export { Adsr } from '../bindings/Adsr';
export { Tap } from '../bindings/Tap';

export { SynthMessage } from '../bindings/SynthMessage';
import { SynthMessage } from '../bindings/SynthMessage';

export type MeterData = Omit<SynthMessage & { t: 'meter' }, 't'>;
