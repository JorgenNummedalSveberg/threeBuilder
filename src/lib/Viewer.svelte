<script lang="ts">
    import { onMount } from "svelte";
    import * as THREE from "three";
    import { Brush, Evaluator, SUBTRACTION } from "three-bvh-csg";
    import { OrbitControls } from "three/addons/controls/OrbitControls.js";
    import init, { parse_bem } from "../../parser/pkg";
    import { pallet } from "./pallet";
    import { type Surface } from "./types";

    let viewer: HTMLCanvasElement;
    let scene: THREE.Scene;
    type Camera = THREE.PerspectiveCamera & {
        position: { x: number; y: number; z: number };
    };
    let camera: Camera;
    let controls: OrbitControls;
    const animations = [];
    let raycaster: THREE.Raycaster;

    // replace y with z
    onMount(async () => {
        await init();
        viewer.height = window.innerHeight;
        viewer.width = window.innerWidth;
        scene = new THREE.Scene();
        raycaster = new THREE.Raycaster();
        camera = new THREE.PerspectiveCamera(
            75,
            window.innerWidth / window.innerHeight,
            0.1,
            1000,
        ) as Camera;
        controls = new OrbitControls(camera, viewer);
        const renderer = new THREE.WebGLRenderer({
            canvas: viewer,
            logarithmicDepthBuffer: true,
        });
        camera.position.z = 100;
        camera.position.y = 100;
        animations.push(() => {
            controls.update();
        });

        const grid = new THREE.GridHelper(1000, 100);
        scene.add(grid);

        function animate() {
            requestAnimationFrame(animate);
            for (const animation of animations) {
                animation();
            }
            renderer.render(scene, camera);
        }

        animate();

        viewer.addEventListener("click", (event) => {
            const mouse = new THREE.Vector2();
            mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
            mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;
            raycaster.setFromCamera(mouse, camera);
            const intersects = raycaster.intersectObjects(scene.children);
            if (intersects.length > 0) {
                const object = intersects[0].object;
                console.log(object.userData);
            }
        });

        window.addEventListener("resize", () => {
            camera.aspect = window.innerWidth / window.innerHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(window.innerWidth, window.innerHeight);
        });
    });

    let gbXML = null;
    const uploadGBXML = async () => {
        const file = await new Promise((resolve) => {
            const input = document.createElement("input");
            input.type = "file";
            input.accept = ".xml";
            input.onchange = () => {
                resolve(input.files[0]);
            };
            input.click();
        });
        gbXML = await file.text();
    };

    const loadGBXML = async () => {
        document.body.style.cursor = "wait";
        const result: { surfaces: Surface[] } = parse_bem(gbXML);
        console.log(result);
        result.surfaces.forEach((surface) => {
            addSurface(surface);
        });
        // const buidlings: Buidling[] = parse_bem(gbXML);

        // buidlings.forEach((building) => {
        //     building.spaces.forEach((space: Space) => {
        //         // space.shell_geometry.closed_shell.faces.forEach((face) => {
        //         //     const vertices: number[][] = face.vertices;
        //         //     const verticesArray = new Float32Array(
        //         //         vertices.flatMap((vertex) => [
        //         //             vertex[0],
        //         //             vertex[1],
        //         //             vertex[2],
        //         //         ]),
        //         //     );
        //         //     const geometry = new THREE.BufferGeometry();
        //         //     geometry.setAttribute(
        //         //         "position",
        //         //         new THREE.BufferAttribute(verticesArray, 3),
        //         //     );
        //         //     geometry.setIndex(face.index);
        //         //     geometry.computeVertexNormals();
        //         //     const mesh = new THREE.Mesh(
        //         //         geometry,
        //         //         new THREE.MeshBasicMaterial({
        //         //             color: Math.random() * 0xffffff,
        //         //             side: THREE.DoubleSide,
        //         //         }),
        //         //     );
        //         //     scene.add(mesh);
        //         // });

        //         const material = new THREE.MeshBasicMaterial({
        //             color: Math.random() * 0xffffff,
        //             side: THREE.DoubleSide,
        //             transparent: true,
        //             opacity: 0.5,
        //         });
        //         const geometries = [];
        //         space.shell_geometry.analytical_shell.shell_surfaces.forEach(
        //             (shell_surface) => {
        //                 shell_surface.faces.forEach((face) => {
        //                     const vertices: number[][] = face.vertices;
        //                     const verticesArray = new Float32Array(
        //                         vertices.flatMap((vertex) => [
        //                             vertex[0],
        //                             vertex[1],
        //                             vertex[2],
        //                         ]),
        //                     );
        //                     const geometry = new THREE.BufferGeometry();
        //                     geometry.setAttribute(
        //                         "position",
        //                         new THREE.BufferAttribute(verticesArray, 3),
        //                     );
        //                     geometry.setIndex(face.index);
        //                     geometries.push(geometry);
        //                 });
        //             },
        //         );
        //         const geometry =
        //             BufferGeometryUtils.mergeGeometries(geometries);
        //         geometry.computeVertexNormals();
        //         const mesh = new THREE.Mesh(geometry, material);
        //         mesh.userData = space;
        //         scene.add(mesh);
        //     });
        // });
        document.body.style.cursor = "default";
    };

    const addSubtraction = (
        geometry1,
        geometry2: THREE.BufferGeometry[],
        surface_type,
    ) => {
        const evaluator = new Evaluator();
        const brush1 = new Brush(geometry1);
        let result;
        for (const geometry of geometry2) {
            const brush2 = new Brush(geometry);
            result = evaluator.evaluate(brush1, brush2, SUBTRACTION);
            brush1.geometry = result.geometry;
        }
        const mesh = new THREE.Mesh(result.geometry, getMaterial(surface_type));
        scene.add(mesh);
    };

    const getMaterial = (surface_type) => {
        return new THREE.MeshBasicMaterial({
            color: pallet[surface_type].color,
            side: THREE.DoubleSide,
            blending: THREE.NoBlending,
        });
    };

    const addSurface = (surface: Surface) => {
        const surfaceVertices = new Float32Array(
            surface.model.vertices.flatMap((vertex) => [
                vertex.x,
                vertex.y,
                vertex.z,
            ]),
        );

        const surfaceGeometry = new THREE.BufferGeometry();
        surfaceGeometry.setAttribute(
            "position",
            new THREE.BufferAttribute(surfaceVertices, 3),
        );
        surfaceGeometry.setIndex(surface.model.faces);
        surfaceGeometry.computeVertexNormals();

        surfaceGeometry.setAttribute(
            "uv",
            new THREE.BufferAttribute(
                new Float32Array((surfaceVertices.length / 3) * 2),
                2,
            ),
        );

        if (surface.openings.length === 0) {
            const material = new THREE.MeshBasicMaterial({
                color: pallet[surface.surface_type].color,
                side: THREE.DoubleSide,
            });
            const mesh = new THREE.Mesh(surfaceGeometry, material);
            scene.add(mesh);
        } else {
            const openingGeometries = [];
            for (const opening of surface.openings) {
                const vertices = new Float32Array(
                    opening.model.vertices.flatMap((vertex) => [
                        vertex.x,
                        vertex.y,
                        vertex.z,
                    ]),
                );
                const geometry = new THREE.BufferGeometry();
                geometry.setAttribute(
                    "position",
                    new THREE.BufferAttribute(vertices, 3),
                );
                geometry.setIndex(opening.model.faces);
                geometry.computeVertexNormals();

                geometry.setAttribute(
                    "uv",
                    new THREE.BufferAttribute(
                        new Float32Array((vertices.length / 3) * 2),
                        2,
                    ),
                );

                openingGeometries.push(geometry);
            }
            openingGeometries.forEach((element) => {
                const transparentMaterial = new THREE.MeshBasicMaterial({
                    color: pallet[surface.surface_type].color,
                    side: THREE.DoubleSide,
                    transparent: true,
                    opacity: 0.5,
                });
                const mesh = new THREE.Mesh(element, transparentMaterial);
                scene.add(mesh);
            });
            addSubtraction(
                surfaceGeometry,
                openingGeometries,
                surface.surface_type,
            );
        }
    };
</script>

<main>
    <canvas bind:this={viewer}></canvas>
    <div class="controls">
        <button on:click={uploadGBXML}>Upload gbXML</button>
        {#if gbXML}
            <button on:click={loadGBXML}>Load gbXML</button>
        {/if}
    </div>
</main>

<style>
    main {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 100vh;
    }
    canvas {
        width: 100%;
        height: 100%;
    }
    .controls {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        display: flex;
        justify-content: center;
        padding: 1rem;
        background-color: rgba(0, 0, 0, 0.5);
    }
</style>
