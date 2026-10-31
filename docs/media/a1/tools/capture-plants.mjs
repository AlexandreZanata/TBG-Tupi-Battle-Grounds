import { connect } from './tbg-a1-cdp.mjs';

async function main() {
  const cdp = await connect(9333);
  console.log('Connected to Chrome on 9333');

  // Bloomkeeper Gallery
  console.log('Navigating to Bloomkeeper Gallery...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=bloomkeeper' });
  await new Promise(r => setTimeout(r, 2500));
  await cdp.screenshot('docs/media/a1/plants-bloomkeeper-gallery.jpg');
  console.log('Saved docs/media/a1/plants-bloomkeeper-gallery.jpg');

  // Bloomkeeper Silhouette
  console.log('Navigating to Bloomkeeper Silhouette...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=bloomkeeper&lineup&silhouette' });
  await new Promise(r => setTimeout(r, 2000));
  await cdp.screenshot('docs/media/a1/plants-bloomkeeper-silhouette.jpg');
  console.log('Saved docs/media/a1/plants-bloomkeeper-silhouette.jpg');

  // Rootcaller Gallery
  console.log('Navigating to Rootcaller Gallery...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=rootcaller' });
  await new Promise(r => setTimeout(r, 2500));
  await cdp.screenshot('docs/media/a1/plants-rootcaller-gallery.jpg');
  console.log('Saved docs/media/a1/plants-rootcaller-gallery.jpg');

  // Rootcaller Silhouette
  console.log('Navigating to Rootcaller Silhouette...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=rootcaller&lineup&silhouette' });
  await new Promise(r => setTimeout(r, 2000));
  await cdp.screenshot('docs/media/a1/plants-rootcaller-silhouette.jpg');
  console.log('Saved docs/media/a1/plants-rootcaller-silhouette.jpg');

  // Nester Gallery
  console.log('Navigating to Nester Gallery...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=nester' });
  await new Promise(r => setTimeout(r, 2500));
  await cdp.screenshot('docs/media/a1/plants-nester-gallery.jpg');
  console.log('Saved docs/media/a1/plants-nester-gallery.jpg');

  // Nester Silhouette
  console.log('Navigating to Nester Silhouette...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=nester&lineup&silhouette' });
  await new Promise(r => setTimeout(r, 2000));
  await cdp.screenshot('docs/media/a1/plants-nester-silhouette.jpg');
  console.log('Saved docs/media/a1/plants-nester-silhouette.jpg');

  // Beetling Gallery
  console.log('Navigating to Beetling Gallery...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=beetling' });
  await new Promise(r => setTimeout(r, 2500));
  await cdp.screenshot('docs/media/a1/plants-beetling-gallery.jpg');
  console.log('Saved docs/media/a1/plants-beetling-gallery.jpg');

  // Beetling Silhouette
  console.log('Navigating to Beetling Silhouette...');
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery&unit=beetling&lineup&silhouette' });
  await new Promise(r => setTimeout(r, 2000));
  await cdp.screenshot('docs/media/a1/plants-beetling-silhouette.jpg');
  console.log('Saved docs/media/a1/plants-beetling-silhouette.jpg');

  cdp.close();
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
