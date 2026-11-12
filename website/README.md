# Citta landing page

Production: https://citta-kappa.vercel.app

Static HTML, CSS and JavaScript. No framework build, server, analytics tracker or application secrets. The demo iframe loads only after Play is clicked.

## Preview

From the repository root:

```sh
python3 -m http.server 8765 --bind 127.0.0.1 --directory website
```

Open http://127.0.0.1:8765. Check desktop and mobile layouts, all platform downloads and checksum links, copy commands, the capability controls, FAQ and video playback.

## Deploy

Vercel project `citta`, scope `priyankarks-projects`, is connected to this repository with production branch `main`, root directory `website`, no build/install command, and output directory `.`. Git pushes create preview deployments; merges to `main` publish production.

For a manual deployment, run from the **repository root** after linking:

```sh
npx vercel link --yes --project citta --scope priyankarks-projects
npx vercel deploy --prod --scope priyankarks-projects
```

The root `.vercelignore` limits CLI uploads to this website. Local Vercel metadata and generated environment files stay ignored. This static project does not need an environment file. Verify the production URL in a signed-out browser; an authenticated CLI request alone does not establish public access.

The website's deployment is separate from the Citta runtime release. Updating a landing page does not qualify or replace a runtime binary.
