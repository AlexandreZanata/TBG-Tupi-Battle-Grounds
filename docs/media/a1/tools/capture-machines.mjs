import { connect } from './tbg-a1-cdp.mjs';

async function main() {
  const cdp = await connect(9333);
  console.log('Connected to Chrome on 9333');

  // Sawbot Gallery
  console.log('Navigating to Sawbot Gallery...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=sawbot' });
  await new Promise(r => setTimeout(r, 2500));
  await cdp.screenshot('docs/media/a1/machines-sawbot-gallery.jpg');
  console.log('Saved docs/media/a1/machines-sawbot-gallery.jpg');

  // Sawbot Silhouette
  console.log('Navigating to Sawbot Silhouette...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=sawbot&lineup&silhouette' });
  await new Promise(r => setTimeout(r, 2000));
  await cdp.screenshot('docs/media/a1/machines-sawbot-silhouette.jpg');
  console.log('Saved docs/media/a1/machines-sawbot-silhouette.jpg');

  // Siege Walker Gallery
  console.log('Navigating to Siege Walker Gallery...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=siegewalker' });
  await new Promise(r => setTimeout(r, 2500));
  await cdp.screenshot('docs/media/a1/machines-siegewalker-gallery.jpg');
  console.log('Saved docs/media/a1/machines-siegewalker-gallery.jpg');

  // Siege Walker Silhouette
  console.log('Navigating to Siege Walker Silhouette...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=siegewalker&lineup&silhouette' });
  await new Promise(r => setTimeout(r, 2000));
  await cdp.screenshot('docs/media/a1/machines-siegewalker-silhouette.jpg');
  console.log('Saved docs/media/a1/machines-siegewalker-silhouette.jpg');

  // Rampart Gallery
  console.log('Navigating to Rampart Gallery...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=rampart' });
  await new Promise(r => setTimeout(r, 2500));
  await cdp.screenshot('docs/media/a1/machines-rampart-gallery.jpg');
  console.log('Saved docs/media/a1/machines-rampart-gallery.jpg');

  // Rampart Silhouette
  console.log('Navigating to Rampart Silhouette...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=rampart&lineup&silhouette' });
  await new Promise(r => setTimeout(r, 2000));
  await cdp.screenshot('docs/media/a1/machines-rampart-silhouette.jpg');
  console.log('Saved docs/media/a1/machines-rampart-silhouette.jpg');

  cdp.close();
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
