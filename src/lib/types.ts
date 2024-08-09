export type Model = {
    vertices: {x: number, y: number, z: number}[]
    faces: Uint32Array;
}

export type Surface = {
    id: string;
    name?: string;
    surface_type: string;
    model: Model;
    openings?: Surface[];
}

export type Structure = {
    surfaces: Surface[];
}

export type Buidling = {
    id: string;
    building_type: string;
    area: number;
    spaces: Space[];
}

export type Space = {
    id: string;
    zone_id_ref: string;
    name: string;
    area: number;
    volume: number;
    shell_geometry: ShellGeometry;
    cad_object_id: number;
}

export type ShellGeometry = {
    id: string;
    unit: string;
    closed_shell?: ClosedShell;
    analytical_shell?: AnalyticalShell;
}

export type ClosedShell = {
    faces: Face[];
}

export type AnalyticalShell = {
    shell_surfaces: ShellSurface[];
}

export type ShellSurface = {
    surface_type: string;
    faces: Face[];
}

export type Face = {
    vertices: number[][];
    index: number[];
}