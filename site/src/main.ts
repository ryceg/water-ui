import { mount } from 'svelte';
import { configureEngineHost, getEngineHost } from 'water-ui';
import App from './App.svelte';
import './app.css';

// The page shows far more artwork than an app screen would: the default cap of
// four is taken by gallery reveals scrolled past before they finish, which
// leaves the live-only washes and drops further down with nothing to draw.
configureEngineHost({ maxLiveInstances: 8 });
getEngineHost().warmWhenIdle();
mount(App, { target: document.getElementById('app')! });
