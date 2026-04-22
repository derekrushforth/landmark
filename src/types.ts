export interface SpaceInfo {
  id: number;
  index: number;
  uuid: string;
  display_id: string;
  active: boolean;
}

export type NameMap = Record<string, string>;

export interface Space extends SpaceInfo {
  displayName: string;
}
