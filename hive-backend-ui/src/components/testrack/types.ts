export enum PartType {
  RPI,
  PSS,
  TSS,
}

export type RackPart = {
  type: PartType;
  // Location in the overall rack, including RPI and PSS
  location: number;
  // Index in TSS stack (Only used if type is TSS)
  index: number;
};
