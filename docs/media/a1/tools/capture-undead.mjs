import { connect } from './tbg-a1-cdp.mjs';

async function main() {
  const cdp = await connect(9333);
  console.log('Connected to Chrome on 9333');

  // Bone Brute Gallery
  console.log('Navigating to Bone Brute Gallery...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=bonebrute' });
  await new Promise(r => setTimeout(r, 2500));
  await cdp.screenshot('docs/media/a1/undead-bonebrute-gallery.jpg');
  console.log('Saved docs/media/a1/undead-bonebrute-gallery.jpg');

  // Bone Brute Silhouette
  console.log('Navigating to Bone Brute Silhouette...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=bonebrute&lineup&silhouette' });
  await new Promise(r => setTimeout(r, 2000));
  await cdp.screenshot('docs/media/a1/undead-bonebrute-silhouette.jpg');
  console.log('Saved docs/media/a1/undead-bonebrute-silhouette.jpg');

  // Hexer Gallery
  console.log('Navigating to Hexer Gallery...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=hexer' });
  await new Promise(r => setTimeout(r, 2500));
  await cdp.screenshot('docs/media/a1/undead-hexer-gallery.jpg');
  console.log('Saved docs/media/a1/undead-hexer-gallery.jpg');

  // Hexer Silhouette
  console.log('Navigating to Hexer Silhouette...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=hexer&lineup&silhouette' });
  await new Promise(r => setTimeout(r, 2000));
  await cdp.screenshot('docs/media/a1/undead-hexer-silhouette.jpg');
  console.log('Saved docs/media/a1/undead-hexer-silhouette.jpg');

  cdp.close();
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
