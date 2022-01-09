import { Global } from '@emotion/react';
import CssBaseline from '@mui/material/CssBaseline';
import { ThemeProvider } from '@mui/material/styles';
import PropTypes from 'prop-types';
import * as React from 'react';
import { Helmet } from 'react-helmet';
import { muiTheme, globals } from '@discretize/react-discretize-components';

export default function TopLayout(props) {
  const title = 'Discretize - Gear Optimizer';
  const description =
    'The Gear Optimizer helps players of the MMORPG Guild Wars 2 to find the most optimal builds for their favourit content while respecting input parameters.';
  const url = process.env.GATSBY_SITE_URL;

  return (
    <React.Fragment>
      <Helmet>
        <title>{title}</title>
        <meta name="description" content={description} />

        <link rel="canonical" href={url} />

        <meta itemProp="name" content={title} />
        <meta itemProp="description" content={description} />

        <meta property="og:url" content={url} />
        <meta property="og:title" content={title} />
        <meta property="og:description" content={description} />

        <meta name="viewport" content="initial-scale=1, width=device-width" />
        <meta name="language" content="english" />
        <meta httpEquiv="content-language" content="en" />
      </Helmet>
      <Global styles={globals} />

      <ThemeProvider theme={muiTheme}>
        <CssBaseline />
        {props.children}
      </ThemeProvider>
    </React.Fragment>
  );
}

TopLayout.propTypes = {
  children: PropTypes.node,
};
