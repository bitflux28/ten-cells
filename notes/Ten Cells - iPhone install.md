---
tags: [ten-cells, game, iphone]
date: 2026-11-25
version: 0.14.1
---

# Ten Cells: iPhone install (0.14.1)

Made Ten Cells easier to get onto an iPhone.

## What changed
- When you open the game in Safari, it shows a small bar: tap Share, then Add to Home Screen. Tap ✕ and it won't show again.
- Stats › Settings has a new **iPhone app** section. In Safari it lists the 3 steps. Inside the Claude app it explains how to put the game online first.
- The iPhone zip now has its files at the top level, so hosts serve the game right at the link.
- Install guide now uses Vercel Drop (vercel.com/drop), which lets you pick the zip straight from Files on the phone. Netlify Drop is the backup.

## Quickest: the profile (like Extreme Odyssey)
The game is now public at https://bitflux28.github.io/ten-cells/ (GitHub Pages, repo bitflux28/ten-cells, branch gh-pages). No Claude login.
1. Open `TenCells-iPhone.mobileconfig` on the iPhone (if nothing happens, save it to Files and tap it there).
2. Settings › Profile Downloaded › Install. It says "Unsigned"; that's normal for a homemade file.
3. Open Ten Cells from the new icon.
- Works offline after the first open. Updates: Claude pushes to the gh-pages branch and the app updates itself.
- Separate save from the Claude version: Export there, Import here once.
- Remove it by deleting the icon, or Settings › General › VPN & Device Management.

## Offline version: the zip
1. Save `Ten Cells web app (iPhone).zip` to Files. Don't unzip it.
2. Safari › vercel.com/drop › choose the zip.
3. Claim Project with a free account, or the link expires.
4. Open the .vercel.app link › Share › Add to Home Screen.

## Things to remember
- Safari, the Claude app and the home screen app each keep their own save. Move progress with Export and Import.
- To update: upload the new zip on the same Vercel project. Same link, so the save stays.
- Works offline once it's on the home screen.
