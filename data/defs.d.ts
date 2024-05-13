export type Games = Game[];

export interface Game {
	name: string;
	yno_translation?: string;
	patches: Patch[];
}

export interface Patch {
	standalone?: false;
	version: [major: number, minor: number, patch: number];
	prerelease?: [kind: string, rev: number];
	version_name: string;
	link: string;
	path: string;
}
