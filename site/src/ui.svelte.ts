import { setPresentation, type PaletteId, type Presentation, type Surface } from 'water-ui';

class Ui {
  surface: Surface = $state(document.documentElement.classList.contains('dark') ? 'dark' : 'light');
  presentation: Presentation = $state('animated');
  palette: PaletteId | undefined = $state();

  setSurface(surface: Surface) {
    this.surface = surface;
    document.documentElement.classList.toggle('dark', surface === 'dark');
  }

  setPresentation(presentation: Presentation) {
    this.presentation = presentation;
    setPresentation(presentation);
  }
}

export const ui = new Ui();
