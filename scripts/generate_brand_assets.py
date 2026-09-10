import os
import numpy as np
from PIL import Image, ImageFilter

SOURCE_PATH = r"C:\Users\rafae\.gemini\antigravity-ide\brain\b19c62b9-9636-40ba-a657-f98b47644b83\.user_uploaded\media_1789078264339.jpg"

def extract_rgba(crop_arr, bg_color, threshold=10.0, ramp=24.0, dilate_radius=5, cutoff=0.03):
    diff = np.linalg.norm(crop_arr - bg_color, axis=2)
    alpha_raw = np.clip((diff - threshold) / ramp, 0.0, 1.0)
    
    # Core detection
    core = (diff > threshold + ramp * 0.4).astype(np.uint8) * 255
    if dilate_radius > 0:
        core_pil = Image.fromarray(core).filter(ImageFilter.MaxFilter(dilate_radius * 2 + 1))
        core_dilated = np.array(core_pil) > 0
    else:
        core_dilated = core > 0
        
    alpha = np.where(core_dilated, alpha_raw, 0.0)
    alpha = np.where(alpha < cutoff, 0.0, alpha)
    
    # Unmultiply background color
    fg = np.zeros_like(crop_arr)
    mask = alpha > 0.01
    for c in range(3):
        fg[:, :, c] = np.where(
            mask,
            np.clip((crop_arr[:, :, c] - (1.0 - alpha) * bg_color[c]) / np.maximum(alpha, 0.01), 0, 255),
            0
        )
        
    rgba = np.zeros((crop_arr.shape[0], crop_arr.shape[1], 4), dtype=np.uint8)
    rgba[:, :, :3] = fg.astype(np.uint8)
    rgba[:, :, 3] = (alpha * 255).astype(np.uint8)
    return Image.fromarray(rgba)

def make_square_icon(sym_img, canvas_size, sym_width_ratio=0.84):
    sym_w = int(round(canvas_size * sym_width_ratio))
    sym_h = int(round(sym_w * sym_img.height / sym_img.width))
    
    # In case height exceeds target ratio
    if sym_h > canvas_size * 0.90:
        sym_h = int(round(canvas_size * 0.90))
        sym_w = int(round(sym_h * sym_img.width / sym_img.height))
        
    resized_sym = sym_img.resize((sym_w, sym_h), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
    x_pos = (canvas_size - sym_w) // 2
    y_pos = (canvas_size - sym_h) // 2
    canvas.paste(resized_sym, (x_pos, y_pos), resized_sym)
    return canvas

def make_padded_logo(logo_img, pad=48):
    canvas = Image.new("RGBA", (logo_img.width + pad * 2, logo_img.height + pad * 2), (0, 0, 0, 0))
    canvas.paste(logo_img, (pad, pad), logo_img)
    return canvas

def main():
    print("Loading official source image...")
    src = Image.open(SOURCE_PATH)
    arr = np.array(src, dtype=float)
    
    bg_dark = np.array([22.5, 23.0, 31.0])
    bg_light = np.array([246.0, 247.0, 250.0])
    bg_gray = np.array([129.0, 130.0, 132.0])
    
    # 1. Extract large symbol (top-left, highest resolution)
    print("Extracting high-resolution symbol...")
    crop_sym = arr[75:285, 70:385]
    sym_rgba = extract_rgba(crop_sym, bg_dark, threshold=10.0, ramp=24.0, dilate_radius=5)
    sym_bbox = sym_rgba.getbbox()
    sym_tight = sym_rgba.crop(sym_bbox)
    print(f"High-res symbol tight bounds: {sym_tight.size}")
    
    # 2. Extract top-right logo (Symbol + Nexa)
    print("Extracting top-right logo...")
    crop_logo = arr[95:208, 470:935]
    logo_rgba = extract_rgba(crop_logo, bg_dark, threshold=11.0, ramp=24.0, dilate_radius=5)
    logo_bbox = logo_rgba.getbbox()
    logo_tight = logo_rgba.crop(logo_bbox)
    print(f"Logo tight bounds: {logo_tight.size}")
    
    # Separate symbol and text components
    # Symbol is on the left, text starts around x=162
    sym_crop_w = 162
    sym_part = logo_tight.crop((0, 0, sym_crop_w, logo_tight.height))
    text_part = logo_tight.crop((sym_crop_w, 0, logo_tight.width, logo_tight.height))
    
    # 3. Extract Card 3 symbol (light background)
    print("Extracting light variant symbol...")
    crop_s3 = arr[385:465, 340:455]
    s3_rgba = extract_rgba(crop_s3, bg_light, threshold=15.0, ramp=28.0, dilate_radius=3)
    s3_bbox = s3_rgba.getbbox()
    s3_tight = s3_rgba.crop(s3_bbox)
    print(f"Card 3 symbol tight bounds: {s3_tight.size}")
    
    # 4. Extract Card 4 symbol (monochrome variant)
    print("Extracting monochrome variant symbol...")
    crop_s4 = arr[385:465, 480:595]
    s4_rgba = extract_rgba(crop_s4, bg_gray, threshold=14.0, ramp=28.0, dilate_radius=3)
    s4_bbox = s4_rgba.getbbox()
    s4_tight = s4_rgba.crop(s4_bbox)
    print(f"Card 4 symbol tight bounds: {s4_tight.size}")
    
    # 5. Build full logo variants at 2x crisp resolution
    scale_factor = 2
    logo_2x_w = logo_tight.width * scale_factor
    logo_2x_h = logo_tight.height * scale_factor
    
    # --- LOGO FULL / DARK ---
    logo_dark_base = logo_tight.resize((logo_2x_w, logo_2x_h), Image.Resampling.LANCZOS)
    logo_full = make_padded_logo(logo_dark_base, pad=48)
    logo_dark = make_padded_logo(logo_dark_base, pad=48)
    
    # --- LOGO LIGHT ---
    # Match Card 3 symbol to sym_part dimensions
    s3_matched = s3_tight.resize(sym_part.size, Image.Resampling.LANCZOS)
    
    # Brand dark slate text [22, 23, 31] with preserved anti-aliasing
    text_arr = np.array(text_part)
    dark_text_arr = text_arr.copy()
    dark_text_arr[:, :, 0] = 22
    dark_text_arr[:, :, 1] = 23
    dark_text_arr[:, :, 2] = 31
    dark_text = Image.fromarray(dark_text_arr)
    
    logo_light_base = Image.new("RGBA", logo_tight.size, (0, 0, 0, 0))
    logo_light_base.paste(s3_matched, (0, 0), s3_matched)
    logo_light_base.paste(dark_text, (sym_crop_w, 0), dark_text)
    logo_light_2x = logo_light_base.resize((logo_2x_w, logo_2x_h), Image.Resampling.LANCZOS)
    logo_light = make_padded_logo(logo_light_2x, pad=48)
    
    # --- LOGO MONOCHROME ---
    # Match Card 4 symbol to sym_part dimensions
    s4_matched = s4_tight.resize(sym_part.size, Image.Resampling.LANCZOS)
    logo_mono_base = Image.new("RGBA", logo_tight.size, (0, 0, 0, 0))
    logo_mono_base.paste(s4_matched, (0, 0), s4_matched)
    logo_mono_base.paste(text_part, (sym_crop_w, 0), text_part)
    logo_mono_2x = logo_mono_base.resize((logo_2x_w, logo_2x_h), Image.Resampling.LANCZOS)
    logo_mono = make_padded_logo(logo_mono_2x, pad=48)
    
    # 6. Generate square icons
    print("Generating square icons at all standard sizes...")
    # Base 1024 icon:
    icon_1024 = make_square_icon(sym_tight, 1024, sym_width_ratio=0.84)
    
    icons = {}
    for sz in [1024, 512, 256, 128, 64, 32, 16]:
        if sz == 1024:
            icons[sz] = icon_1024
        else:
            # High-quality Lanczos downsampling from 1024
            ratio = 0.84 if sz >= 32 else 0.88
            icons[sz] = make_square_icon(sym_tight, sz, sym_width_ratio=ratio)
            
    # Output directories
    dirs = [
        r"c:\xampp\htdocs\deskflow-master\nexa\assets\brand",
        r"c:\xampp\htdocs\deskflow-master\nexa\assets\icons",
        r"c:\xampp\htdocs\deskflow-master\nexa\assets\images",
    ]
    for d in dirs:
        os.makedirs(d, exist_ok=True)
        
    deliverables = {
        "nexa-icon-16.png": icons[16],
        "nexa-icon-32.png": icons[32],
        "nexa-icon-64.png": icons[64],
        "nexa-icon-128.png": icons[128],
        "nexa-icon-256.png": icons[256],
        "nexa-icon-512.png": icons[512],
        "nexa-icon-1024.png": icons[1024],
        "nexa-logo-full.png": logo_full,
        "nexa-logo-dark.png": logo_dark,
        "nexa-logo-light.png": logo_light,
        "nexa-logo-monochrome.png": logo_mono,
    }
    
    brand_dir = r"c:\xampp\htdocs\deskflow-master\nexa\assets\brand"
    icons_dir = r"c:\xampp\htdocs\deskflow-master\nexa\assets\icons"
    images_dir = r"c:\xampp\htdocs\deskflow-master\nexa\assets\images"
    
    # Save all 11 files in assets/brand
    for name, img in deliverables.items():
        out_p = os.path.join(brand_dir, name)
        img.save(out_p, "PNG")
        print(f"Saved: {out_p} ({img.size})")
        
    # Also update/replace assets/icons and assets/images
    for sz in [16, 32, 64, 128, 256, 512, 1024]:
        name = f"nexa-icon-{sz}.png"
        icons[sz].save(os.path.join(icons_dir, name), "PNG")
        
    # Generate 48x48 icon for Windows .ico compatibility
    icon_48 = make_square_icon(sym_tight, 48, sym_width_ratio=0.84)
    icon_48.save(os.path.join(icons_dir, "nexa-icon-48.png"), "PNG")
    
    # Generate multi-size ICO files
    ico_sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    ico_imgs = [icons[s[0]] if s[0] != 48 else icon_48 for s in ico_sizes]
    
    fav_ico_path = os.path.join(icons_dir, "favicon.ico")
    win_ico_path = os.path.join(icons_dir, "windows.ico")
    icons[256].save(fav_ico_path, format="ICO", sizes=ico_sizes)
    icons[256].save(win_ico_path, format="ICO", sizes=ico_sizes)
    print(f"Saved ICO files: {fav_ico_path}, {win_ico_path}")
    
    # Transparent symbol reference in icons_dir
    sym_tight.save(os.path.join(icons_dir, "nexa-symbol-transparent.png"), "PNG")
    
    # Variants in icons_dir
    s3_tight.save(os.path.join(icons_dir, "nexa-variant-light.png"), "PNG")
    s4_tight.save(os.path.join(icons_dir, "nexa-variant-mono.png"), "PNG")
    sym_part.save(os.path.join(icons_dir, "nexa-variant-dark.png"), "PNG")
    
    # Save logos in assets/images
    logo_full.save(os.path.join(images_dir, "nexa-logo-full.png"), "PNG")
    logo_dark.save(os.path.join(images_dir, "nexa-logo-dark.png"), "PNG")
    logo_light.save(os.path.join(images_dir, "nexa-logo-light.png"), "PNG")
    logo_mono.save(os.path.join(images_dir, "nexa-logo-monochrome.png"), "PNG")
    logo_full.save(os.path.join(images_dir, "nexa-header-logo.png"), "PNG")
    logo_full.save(os.path.join(images_dir, "nexa_logo_horizontal.png"), "PNG")
    
    # Also save dark squircle app icon for cross-platform app usage (macOS/iOS/Android/Windows tile)
    squircle_canvas = Image.new("RGBA", (512, 512), (0, 0, 0, 0))
    # Draw rounded dark background
    from PIL import ImageDraw
    draw = ImageDraw.Draw(squircle_canvas)
    draw.rounded_rectangle([0, 0, 512, 512], radius=112, fill=(22, 23, 31, 255))
    squircle_sym = make_square_icon(sym_tight, 512, sym_width_ratio=0.76)
    squircle_canvas.paste(squircle_sym, (0, 0), squircle_sym)
    squircle_canvas.save(os.path.join(icons_dir, "nexa-app-icon-squircle.png"), "PNG")
    
    print("\nAll brand assets successfully generated and deployed!")

if __name__ == "__main__":
    main()
