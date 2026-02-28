//! Inspired by Ken Perlin 3D improved noise, but adapted for 2D

/// Get perlin noise value at point (x, y)
pub fn perlin_noise(x: f64, y: f64) -> f64 {
    // Coordinates of the corners of the cell where the noise is computed
    let cell_x0 = x.floor();
    let cell_x1 = cell_x0 + 1.0;
    let cell_y0 = y.floor();
    let cell_y1 = cell_y0 + 1.0;

    // Vectors from the point where the noise is computed, to each corner of the cell
    let v_x0 = cell_x0 - x;
    let v_x1 = cell_x1 - x;
    let v_y0 = cell_y0 - y;
    let v_y1 = cell_y1 - y;

    // Compute the dot product at each corner of the cell, between:
    // - The gradient vector at this corner
    // - The vector from the point where the noise is computed to this corner
    //
    //  A ------- B
    //  |         |
    //  |  .(x,y) |
    //  |         |
    //  C ------- D
    //
    let a = gradient_dot_v(hash(cell_x0, cell_y0), v_x0, v_y0);
    let b = gradient_dot_v(hash(cell_x1, cell_y0), v_x1, v_y0);
    let c = gradient_dot_v(hash(cell_x0, cell_y1), v_x0, v_y1);
    let d = gradient_dot_v(hash(cell_x1, cell_y1), v_x1, v_y1);

    // Coordinates of the evaluated point relative to the top-left corner of the cell
    let r_x0 = x - cell_x0;
    let r_y0 = y - cell_y0;

    // Interpolate to get the perlin noise value at the point (x, y)
    let i_x0 = interpolate(a, b, r_x0);
    let i_x1 = interpolate(c, d, r_x0);
    interpolate(i_x0, i_x1, r_y0)
}

// t must be in [0; 1]
fn interpolate(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * quintic_smoothstep(t)
}

// t must be in [0; 1]
fn quintic_smoothstep(t: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn gradient_dot_v(
    // In [0; 255]
    corner_hash: usize,
    // Vector from one of the corners of the cell to the point where the noise function is computed
    v_x: f64,
    v_y: f64,
) -> f64 {
    // I map h to 4 possible gradient vectors, I hope it is enough
    // (It's analogous to the 3D algorithm which uses 12 directions)
    let h = corner_hash & 3;
    if h == 0 {
        v_x + v_y
    } else if h == 1 {
        -v_x + v_y
    } else if h == 2 {
        v_x - v_y
    } else {
        -v_x - v_y
    }
}

fn hash(x: f64, y: f64) -> usize {
    static P: [usize; 512] = init_p();
    P[P[hash_to_u8(x)] + hash_to_u8(y)]
}

fn hash_to_u8(f: f64) -> usize {
    let mut bits = f.to_bits();
    bits ^= bits >> 32;
    bits ^= bits >> 16;
    bits ^= bits >> 8;
    (bits as usize) & 255
}

const fn init_p() -> [usize; 512] {
    static PERMUTATIONS: [usize; 256] = [
        151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30,
        69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94,
        252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171,
        168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60,
        211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1,
        216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86,
        164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118,
        126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170,
        213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39,
        253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34,
        242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49,
        192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254,
        138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
    ];
    let mut p: [usize; 512] = [0; 512];
    let mut i = 0;
    while i < 256 {
        p[i] = PERMUTATIONS[i];
        p[i + 256] = PERMUTATIONS[i];
        i += 1;
    }
    p
}
